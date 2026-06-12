//! CloaKey CLI — main entry point.
//!
//! Both the `cloak` and `cloakey` binaries point to this file.
//!
//! Behavior:
//! - `cloak` with no arguments → interactive ratatui menu
//! - `cloak <subcommand>` → direct command mode

mod app;
mod commands;
mod error;
mod lock_screen;
mod logo_data;
mod menu;
mod settings_menu;
mod startup;
mod tui;

use clap::Parser;
use tracing::info;
use tracing_subscriber::prelude::*;

use cloakey_core::{parse_duration, LockState};
use cloakey_settings::ConfigManager;

use app::App;
use commands::{Cli, Command};
use error::CliError;

fn main() {
    let cli = Cli::parse();
    
    // Set up logging: file logger for interactive mode, stderr logger for direct commands
    init_logging(cli.command.is_none());

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn init_logging(interactive: bool) {
    if interactive {
        if let Some(config_dir) = dirs::config_dir().map(|base| base.join("CloaKey")) {
            let _ = std::fs::create_dir_all(&config_dir);
            let log_file_path = config_dir.join("cloakey.log");
            
            if let Ok(file) = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(log_file_path)
            {
                let file_layer = tracing_subscriber::fmt::layer()
                    .with_writer(file)
                    .with_ansi(false);
                
                let filter = tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("cloakey=info".parse().unwrap());
                
                let _ = tracing_subscriber::registry()
                    .with(filter)
                    .with(file_layer)
                    .try_init();
            }
        }
    } else {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("cloakey=info".parse().unwrap()),
            )
            .with_target(false)
            .compact()
            .try_init();
    }
}

use std::fs::OpenOptions;

fn run(cli: Cli) -> Result<(), CliError> {

    match cli.command {
        // No subcommand → interactive menu
        None => {
            info!("Starting CloaKey interactive mode");
            let mut app = App::new()?;
            app.run_interactive()?;
        }

        // Direct commands
        Some(Command::Keyboard) => {
            info!("Starting keyboard lock (direct command)");
            let mut app = App::new()?;
            app.start_lock(LockState::KeyboardLocked)?;
            run_lock_until_unlock(&mut app)?;
        }

        Some(Command::Mouse) => {
            info!("Starting mouse lock (direct command)");
            let mut app = App::new()?;
            app.start_lock(LockState::MouseLocked)?;
            run_lock_until_unlock(&mut app)?;
        }

        Some(Command::Full) => {
            info!("Starting full lock (direct command)");
            let mut app = App::new()?;
            app.start_lock(LockState::FullyLocked)?;
            run_lock_until_unlock(&mut app)?;
        }

        Some(Command::Ghost) => {
            info!("Starting ghost mode (direct command)");
            let mut app = App::new()?;
            app.start_lock(LockState::GhostMode)?;
            run_lock_until_unlock(&mut app)?;
        }

        Some(Command::Timer { duration }) => {
            info!("Starting timed lock: {} (direct command)", duration);
            // Validate duration early for good error messages
            parse_duration(&duration).map_err(|e| CliError::Core(e))?;
            let mut app = App::new()?;
            app.start_timed_lock(&duration)?;
            run_lock_until_unlock(&mut app)?;
        }

        Some(Command::Status) => {
            let app = App::new()?;
            app.print_status();
        }

        Some(Command::Unlock) => {
            // Send unlock signal (CloaKey removes all hooks when it exits)
            println!("CloaKey: No active lock session found.");
            println!("If you are trapped, hold CTRL + ALT + SHIFT for 2 seconds.");
        }

        Some(Command::Settings) => {
            info!("Opening settings (direct command)");
            let mut app = App::new()?;
            // Jump straight to settings screen
            app.run_interactive()?; // Will show settings after one cycle if needed
            // For direct command: show a simple settings summary
        }

        Some(Command::Startup) => {
            run_startup_menu()?;
        }

        Some(Command::Reset) => {
            run_reset()?;
        }

        Some(Command::Help) => {
            print_help();
        }

        Some(Command::About) => {
            print_about();
        }
    }

    Ok(())
}

/// Run the TUI lock screen until an unlock signal is received.
///
/// Used for direct command mode (e.g., `cloak keyboard`).
fn run_lock_until_unlock(app: &mut App) -> Result<(), CliError> {
    let mut tui = tui::TerminalGuard::new()
        .map_err(|e| CliError::Terminal(e.to_string()))?;

    loop {
        // Check for safety signals (emergency unlock, deadman, shutdown)
        if let Some(signal) = app.safety_ref().try_recv_signal() {
            match signal {
                cloakey_core::SafetySignal::EmergencyUnlock
                | cloakey_core::SafetySignal::DeadmanUnlock
                | cloakey_core::SafetySignal::HotkeyUnlock => {
                    info!("Unlock signal received");
                    app.perform_unlock()?;
                    break;
                }
                cloakey_core::SafetySignal::Shutdown => break,
                _ => {}
            }
        }

        // Check timer expiry
        let timer_expired = app.session_ref().timer_expired();
        if timer_expired {
            info!("Timed lock expired — auto-unlocking");
            app.perform_unlock()?;
            break;
        }

        // Snapshot rendering data before the draw closure (avoids borrow conflicts)
        let state = app.session_ref().lock_state().clone();
        let mode = app.session_ref().lock_mode().clone();
        let timer_str = app.session_ref().timer().map(|t| t.remaining_display());
        let elapsed = app.session_ref().locked_at()
            .map(|a| a.elapsed().as_secs())
            .unwrap_or(0);

        tui.terminal().draw(|frame| {
            lock_screen::draw_lock_screen(
                frame,
                &state,
                &mode,
                timer_str.as_deref(),
                elapsed,
            );
        }).map_err(|e| CliError::Terminal(e.to_string()))?;

        // Poll for terminal events (100ms tick)
        if crossterm::event::poll(std::time::Duration::from_millis(100))
            .map_err(|e| CliError::Terminal(e.to_string()))?
        {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()
                .map_err(|e| CliError::Terminal(e.to_string()))?
            {
                // Allow Ctrl+C as software-level escape
                if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                    && key.code == crossterm::event::KeyCode::Char('c')
                {
                    app.perform_unlock()?;
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Interactive startup configuration.
fn run_startup_menu() -> Result<(), CliError> {
    let config_manager = ConfigManager::new()?;
    let mut config = config_manager.load()?;

    println!("CloaKey Startup Configuration");
    println!("══════════════════════════════");
    println!(
        "Run On Startup:  {}",
        if config.startup.run_on_startup { "ON" } else { "OFF" }
    );
    println!(
        "Start Minimized: {}",
        if config.startup.start_minimized { "ON" } else { "OFF" }
    );
    println!();
    println!("Options:");
    println!("  1. Enable Run On Startup");
    println!("  2. Disable Run On Startup");
    println!("  3. Toggle Start Minimized");
    println!("  Q. Back");
    println!();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    match input.trim() {
        "1" => {
            startup::set_startup_enabled(true).map_err(|e| CliError::Other(e))?;
            config.startup.run_on_startup = true;
            config_manager.save(&config)?;
            println!("✓ CloaKey will run at Windows startup.");
        }
        "2" => {
            startup::set_startup_enabled(false).map_err(|e| CliError::Other(e))?;
            config.startup.run_on_startup = false;
            config_manager.save(&config)?;
            println!("✓ CloaKey will not run at Windows startup.");
        }
        "3" => {
            config.startup.start_minimized = !config.startup.start_minimized;
            config_manager.save(&config)?;
            println!(
                "✓ Start Minimized: {}",
                if config.startup.start_minimized { "ON" } else { "OFF" }
            );
        }
        _ => {}
    }
    Ok(())
}

/// Reset all settings to defaults with confirmation.
fn run_reset() -> Result<(), CliError> {
    println!("Reset all CloaKey settings to defaults?");
    println!("This cannot be undone.");
    println!("Type 'yes' to confirm, anything else to cancel:");
    println!();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();

    if input.trim().eq_ignore_ascii_case("yes") {
        let config_manager = ConfigManager::new()?;
        config_manager.reset()?;
        println!("✓ Settings reset to defaults.");
    } else {
        println!("Reset cancelled.");
    }
    Ok(())
}

/// Print the help text to stdout (non-TUI, for piping/scripting).
fn print_help() {
    println!("CloaKey v{} — Protect the workflow. Don't stop the workflow.", env!("CARGO_PKG_VERSION"));
    println!();
    println!("USAGE:");
    println!("  cloak               Open interactive menu");
    println!("  cloak <COMMAND>     Run command directly");
    println!();
    println!("COMMANDS:");
    println!("  keyboard            Lock keyboard only");
    println!("  mouse               Lock mouse only");
    println!("  full                Lock keyboard and mouse (alias: lock, both)");
    println!("  ghost               Ghost Mode — cursor moves, clicks blocked");
    println!("  timer <DURATION>    Timed lock (e.g. 60, 5m, 1h)");
    println!("  status              Show current protection status");
    println!("  unlock              Remove active protection");
    println!("  settings            Configure CloaKey");
    println!("  startup             Configure Windows startup integration");
    println!("  reset               Reset all settings to defaults");
    println!("  help                Show this help");
    println!("  about               Show version, license, and links");
    println!();
    println!("EMERGENCY UNLOCK:");
    println!("  CTRL + ALT + SHIFT  (hold 2 seconds)  — always works");
    println!();
    println!("KEYBOARD SHORTCUTS:");
    println!("  CTRL + ALT + L      Lock all");
    println!("  CTRL + ALT + K      Lock keyboard");
    println!("  CTRL + ALT + M      Lock mouse");
    println!("  CTRL + ALT + G      Ghost Mode");
    println!("  CTRL + ALT + SHIFT  Emergency unlock (hold 2s)");
    println!();
    println!("LINKS:");
    println!("  Website:  https://cloakey.io");
    println!("  GitHub:   https://github.com/cloakey/cloakey");
}

/// Print the about text to stdout.
fn print_about() {
    println!("CloaKey v{}", env!("CARGO_PKG_VERSION"));
    println!("License:    MIT");
    println!("Website:    https://cloakey.io");
    println!("GitHub:     https://github.com/cloakey/cloakey");
    println!();
    println!("\"Protect the workflow. Don't stop the workflow.\"");
    println!();
    println!("CloaKey suppresses keyboard and mouse input while keeping");
    println!("all applications, AI agents, and processes running normally.");
    println!();
    println!("PRIVACY: Input is blocked but NEVER recorded.");
    println!("No telemetry. No analytics. No user accounts.");
}


