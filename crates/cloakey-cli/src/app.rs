//! Application state and main event loop.
//!
//! `App` is the central coordinator that connects:
//! - Settings (config manager)
//! - Core (session state, timer)
//! - Input engine (hooks)
//! - Overlay (status window)
//! - Safety controller (emergency unlock, deadman switch)
//! - CLI (ratatui rendering, user input)

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::widgets::ListState;
use tracing::{info, warn};

use cloakey_core::{
    parse_duration, CountdownTimer, LockState, SafetyController, SafetySignal, SessionState,
};
use cloakey_input::InputEngine;
use cloakey_overlay::OverlayManager;
use cloakey_settings::{CloaKeyConfig, ConfigManager};

use crate::{
    error::CliError,
    lock_screen::draw_lock_screen,
    menu::{draw_about_screen, draw_help_screen, draw_main_menu, MenuItem},
    settings_menu::{cycle_overlay_mode, draw_settings_menu, SettingsItem},
    startup::set_startup_enabled,
    tui::TerminalGuard,
};

/// Application screen state.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Screen {
    MainMenu,
    LockActive,
    Settings,
    Help,
    About,
    TimedLockPrompt,
}

/// The main CloaKey application.
pub struct App {
    config_manager: ConfigManager,
    config: CloaKeyConfig,
    session: SessionState,
    safety: SafetyController,
    input_engine: Option<InputEngine>,
    overlay: OverlayManager,
    screen: Screen,
    menu_state: ListState,
    settings_state: ListState,
    running: bool,
    timed_lock_input: String,
}

impl App {
    /// Initialize the application.
    pub fn new() -> Result<Self, CliError> {
        let config_manager = ConfigManager::new()?;
        let config = config_manager.load()?;

        let overlay_mode = config.general.overlay.clone();
        let safety = SafetyController::initialize().map_err(|e| CliError::Other(e.to_string()))?;

        // Start the deadman switch background thread
        safety.start_deadman_switch(Duration::from_secs(5));

        let mut menu_state = ListState::default();
        menu_state.select(Some(0));

        let mut settings_state = ListState::default();
        settings_state.select(Some(0));

        Ok(Self {
            config_manager,
            config,
            session: SessionState::new(),
            safety,
            input_engine: None,
            overlay: OverlayManager::new(overlay_mode),
            screen: Screen::MainMenu,
            menu_state,
            settings_state,
            running: true,
            timed_lock_input: String::new(),
        })
    }

    /// Run the interactive TUI event loop.
    pub fn run_interactive(&mut self) -> Result<(), CliError> {
        let mut tui = TerminalGuard::new().map_err(|e| CliError::Terminal(e.to_string()))?;

        while self.running {
            // --- Check for safety signals (emergency unlock, deadman, shutdown) ---
            self.check_safety_signals()?;

            // --- Check timer expiry ---
            if self.session.timer_expired() {
                info!("Timed lock expired — auto-unlocking");
                self.perform_unlock()?;
            }

            // --- Render ---
            let version = env!("CARGO_PKG_VERSION");
            let session_ref = &self.session;
            let screen = self.screen.clone();
            let config_ref = &self.config;
            let menu_state = &mut self.menu_state;
            let settings_state = &mut self.settings_state;
            let timed_input = self.timed_lock_input.clone();

            tui.terminal()
                .draw(|frame| {
                    match screen {
                        Screen::MainMenu => {
                            draw_main_menu(frame, menu_state, session_ref.lock_state(), version);
                        }
                        Screen::LockActive => {
                            let timer_display = session_ref.timer().map(|t| t.remaining_display());
                            let elapsed = session_ref
                                .locked_at()
                                .map(|at| at.elapsed().as_secs())
                                .unwrap_or(0);
                            draw_lock_screen(
                                frame,
                                session_ref.lock_state(),
                                session_ref.lock_mode(),
                                timer_display.as_deref(),
                                elapsed,
                            );
                        }
                        Screen::Settings => {
                            draw_settings_menu(frame, config_ref, settings_state);
                        }
                        Screen::Help => {
                            draw_help_screen(frame);
                        }
                        Screen::About => {
                            draw_about_screen(frame, version);
                        }
                        Screen::TimedLockPrompt => {
                            // Simple timed lock input prompt
                            draw_timed_prompt(frame, &timed_input);
                        }
                    }
                })
                .map_err(|e| CliError::Terminal(e.to_string()))?;

            // --- Handle input events ---
            if event::poll(Duration::from_millis(100))
                .map_err(|e| CliError::Terminal(e.to_string()))?
            {
                let ev = event::read().map_err(|e| CliError::Terminal(e.to_string()))?;
                self.handle_event(ev)?;
            }
        }

        // Clean up before exit
        self.perform_unlock()?;
        Ok(())
    }

    /// Handle a terminal input event.
    fn handle_event(&mut self, event: Event) -> Result<(), CliError> {
        // Only process key PRESS events — ignore Release and Repeat to prevent
        // double-stepping when navigating with arrow keys.
        if let Event::Key(ref key) = event {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }
        }

        match &self.screen {
            Screen::MainMenu => self.handle_menu_event(event),
            Screen::LockActive => self.handle_lock_screen_event(event),
            Screen::Settings => self.handle_settings_event(event),
            Screen::Help | Screen::About => self.handle_info_screen_event(event),
            Screen::TimedLockPrompt => self.handle_timed_prompt_event(event),
        }
    }

    fn handle_menu_event(&mut self, event: Event) -> Result<(), CliError> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.running = false;
                }
                KeyCode::Esc => {
                    self.running = false;
                }
                KeyCode::Down | KeyCode::Tab => {
                    let len = MenuItem::all().len();
                    let i = self.menu_state.selected().unwrap_or(0);
                    self.menu_state.select(Some((i + 1) % len));
                }
                KeyCode::Up => {
                    let len = MenuItem::all().len();
                    let i = self.menu_state.selected().unwrap_or(0);
                    self.menu_state.select(Some((i + len - 1) % len));
                }
                KeyCode::Enter => {
                    if let Some(idx) = self.menu_state.selected() {
                        let item = MenuItem::all()[idx].clone();
                        self.execute_menu_item(item)?;
                    }
                }
                KeyCode::Char(c) => {
                    if let Some(item) = MenuItem::from_key(c) {
                        // Update selection visually first
                        if let Some(idx) = MenuItem::index_from_key(c) {
                            self.menu_state.select(Some(idx));
                        }
                        self.execute_menu_item(item)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_lock_screen_event(&mut self, event: Event) -> Result<(), CliError> {
        // While locked, only the emergency unlock sequence (handled by input engine)
        // can unlock. But we still allow the user to close if they press Ctrl+C
        // from the terminal perspective (since keyboard might be locked by hooks).
        // The TUI screen just shows status — actual unlock comes from safety signals.
        if let Event::Key(key) = event {
            // Allow Ctrl+C as a software escape (hooks may block physical CTRL+C
            // from the terminal, but if it reaches us, honor it)
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                info!("Ctrl+C received on lock screen — unlocking");
                self.perform_unlock()?;
            }
        }
        Ok(())
    }

    fn handle_settings_event(&mut self, event: Event) -> Result<(), CliError> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.screen = Screen::MainMenu;
                }
                KeyCode::Down | KeyCode::Tab => {
                    let len = SettingsItem::all().len();
                    let i = self.settings_state.selected().unwrap_or(0);
                    self.settings_state.select(Some((i + 1) % len));
                }
                KeyCode::Up => {
                    let len = SettingsItem::all().len();
                    let i = self.settings_state.selected().unwrap_or(0);
                    self.settings_state.select(Some((i + len - 1) % len));
                }
                KeyCode::Enter => {
                    if let Some(idx) = self.settings_state.selected() {
                        let item = SettingsItem::all()[idx].clone();
                        self.execute_settings_item(item)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_info_screen_event(&mut self, event: Event) -> Result<(), CliError> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.screen = Screen::MainMenu;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_timed_prompt_event(&mut self, event: Event) -> Result<(), CliError> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Esc => {
                    self.timed_lock_input.clear();
                    self.screen = Screen::MainMenu;
                }
                KeyCode::Backspace => {
                    self.timed_lock_input.pop();
                }
                KeyCode::Enter => {
                    let input = self.timed_lock_input.trim().to_string();
                    self.timed_lock_input.clear();
                    if !input.is_empty() {
                        self.start_timed_lock(&input)?;
                    } else {
                        self.screen = Screen::MainMenu;
                    }
                }
                KeyCode::Char(c) => {
                    self.timed_lock_input.push(c);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn execute_menu_item(&mut self, item: MenuItem) -> Result<(), CliError> {
        match item {
            MenuItem::LockKeyboard => self.start_lock(LockState::KeyboardLocked)?,
            MenuItem::LockMouse => self.start_lock(LockState::MouseLocked)?,
            MenuItem::LockBoth => self.start_lock(LockState::FullyLocked)?,
            MenuItem::GhostMode => self.start_lock(LockState::GhostMode)?,
            MenuItem::TimedLock => {
                self.screen = Screen::TimedLockPrompt;
            }
            MenuItem::Settings => {
                self.screen = Screen::Settings;
            }
            MenuItem::Help => {
                self.screen = Screen::Help;
            }
            MenuItem::About => {
                self.screen = Screen::About;
            }
            MenuItem::Quit => {
                self.running = false;
            }
        }
        Ok(())
    }

    fn execute_settings_item(&mut self, item: SettingsItem) -> Result<(), CliError> {
        match item {
            SettingsItem::OverlayMode => {
                cycle_overlay_mode(&mut self.config);
                self.config_manager.save(&self.config)?;
                // Update overlay manager mode
                let new_mode = self.config.general.overlay.clone();
                self.overlay = OverlayManager::new(new_mode);
            }
            SettingsItem::UnlockHoldDuration => {
                // Cycle through common values: 1000ms, 1500ms, 2000ms, 3000ms
                self.config.shortcuts.emergency_unlock_hold_ms =
                    match self.config.shortcuts.emergency_unlock_hold_ms {
                        1000 => 1500,
                        1500 => 2000,
                        2000 => 3000,
                        _ => 1000,
                    };
                self.config_manager.save(&self.config)?;
            }
            SettingsItem::RunOnStartup => {
                let new_val = !self.config.startup.run_on_startup;
                match set_startup_enabled(new_val) {
                    Ok(()) => {
                        self.config.startup.run_on_startup = new_val;
                        self.config_manager.save(&self.config)?;
                    }
                    Err(e) => {
                        warn!("Failed to update startup setting: {}", e);
                    }
                }
            }
            SettingsItem::StartMinimized => {
                self.config.startup.start_minimized = !self.config.startup.start_minimized;
                self.config_manager.save(&self.config)?;
            }
            SettingsItem::BackToMenu => {
                self.screen = Screen::MainMenu;
            }
        }
        Ok(())
    }

    /// Start a lock in a specific state.
    pub fn start_lock(&mut self, state: LockState) -> Result<(), CliError> {
        // Validate transition
        self.session
            .transition_to(state.clone())
            .map_err(|e| CliError::Core(e))?;

        // Start input engine
        let mode = self.session.lock_mode().clone();
        let hold_ms = self.config.shortcuts.emergency_unlock_hold_ms;
        let signal_tx = self.safety.signal_sender();
        let heartbeat = self.safety.heartbeat();

        let engine = InputEngine::start(mode.clone(), hold_ms, signal_tx, heartbeat)
            .map_err(|e| CliError::Input(e))?;

        self.input_engine = Some(engine);

        // Show overlay
        let timer_display = self.session.timer().map(|t| t.remaining_display());
        self.overlay
            .show(
                &state,
                timer_display.as_deref(),
                self.safety.signal_sender(),
            )
            .map_err(|e| CliError::Overlay(e))?;

        // Minimize the console window while locked
        minimize_console();

        self.screen = Screen::LockActive;
        info!("Lock started: {}", state);
        Ok(())
    }

    /// Start a timed lock.
    pub fn start_timed_lock(&mut self, duration_str: &str) -> Result<(), CliError> {
        let duration = parse_duration(duration_str).map_err(|e| CliError::Core(e))?;
        let timer = CountdownTimer::new(duration);

        self.session
            .start_timed_lock(timer)
            .map_err(|e| CliError::Core(e))?;

        let mode = self.session.lock_mode().clone();
        let hold_ms = self.config.shortcuts.emergency_unlock_hold_ms;
        let signal_tx = self.safety.signal_sender();
        let heartbeat = self.safety.heartbeat();

        let engine = InputEngine::start(mode, hold_ms, signal_tx, heartbeat)
            .map_err(|e| CliError::Input(e))?;

        self.input_engine = Some(engine);

        let timer_display = self.session.timer().map(|t| t.remaining_display());
        self.overlay
            .show(
                &LockState::TimedLock,
                timer_display.as_deref(),
                self.safety.signal_sender(),
            )
            .map_err(|e| CliError::Overlay(e))?;

        self.screen = Screen::LockActive;
        info!("Timed lock started: {:?}", duration);
        Ok(())
    }

    /// Unlock — stop input hooks, hide overlay, restore console, return to menu.
    pub fn perform_unlock(&mut self) -> Result<(), CliError> {
        if let Some(mut engine) = self.input_engine.take() {
            engine.stop();
        }

        self.session.unlock();
        self.overlay.hide();

        // Restore the console window after unlocking
        restore_console();

        self.screen = Screen::MainMenu;

        info!("Unlocked");
        Ok(())
    }

    /// Check for pending safety signals and handle them.
    fn check_safety_signals(&mut self) -> Result<(), CliError> {
        while let Some(signal) = self.safety.try_recv_signal() {
            match signal {
                SafetySignal::EmergencyUnlock => {
                    info!("Emergency unlock signal received — unlocking");
                    self.perform_unlock()?;
                }
                SafetySignal::DeadmanUnlock => {
                    warn!("Deadman switch triggered — forcing unlock for safety");
                    self.perform_unlock()?;
                }
                SafetySignal::HotkeyLockAll => {
                    self.start_lock(LockState::FullyLocked)?;
                }
                SafetySignal::HotkeyLockKeyboard => {
                    self.start_lock(LockState::KeyboardLocked)?;
                }
                SafetySignal::HotkeyLockMouse => {
                    self.start_lock(LockState::MouseLocked)?;
                }
                SafetySignal::HotkeyGhostMode => {
                    self.start_lock(LockState::GhostMode)?;
                }
                SafetySignal::HotkeyUnlock => {
                    info!("Hotkey unlock received — unlocking");
                    self.perform_unlock()?;
                }
                SafetySignal::Shutdown => {
                    info!("Shutdown signal received");
                    self.running = false;
                }
            }
        }
        Ok(())
    }

    /// Print the current status to stdout (for `cloak status` command).
    pub fn print_status(&self) {
        let state = self.session.lock_state();
        println!("CloaKey Status");
        println!("══════════════");
        println!("State:  {}", state);
        println!("Mode:   {}", self.session.status_display());

        if let Some(timer) = self.session.timer() {
            println!("Timer:  {} remaining", timer.remaining_display());
        }

        let shortcuts = &self.config.shortcuts;
        println!(
            "Unlock: {} (hold {}ms)",
            shortcuts.emergency_unlock, shortcuts.emergency_unlock_hold_ms
        );
    }

    /// Get a reference to the session state (for direct command mode).
    pub fn session_ref(&self) -> &SessionState {
        &self.session
    }

    /// Get a reference to the safety controller (for direct command mode).
    pub fn safety_ref(&self) -> &SafetyController {
        &self.safety
    }
}

#[cfg(target_os = "windows")]
fn minimize_console() {
    use windows::Win32::System::Console::GetConsoleWindow;
    use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd.0 as isize != 0 {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn minimize_console() {}

#[cfg(target_os = "windows")]
fn restore_console() {
    use windows::Win32::System::Console::GetConsoleWindow;
    use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOW};
    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd.0 as isize != 0 {
            let _ = ShowWindow(hwnd, SW_SHOW);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn restore_console() {}

/// Simple timed lock duration prompt screen.
fn draw_timed_prompt(frame: &mut ratatui::Frame, input: &str) {
    use ratatui::{
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, BorderType, Borders, Paragraph},
    };

    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    let prompt = Paragraph::new(vec![
        Line::from(Span::styled(
            "Enter lock duration:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Examples: 60  |  5m  |  1h",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  > ", Style::default().fg(Color::Cyan)),
            Span::styled(
                input,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::Cyan)), // cursor
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Enter to confirm · Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .block(
        Block::default()
            .title(" ⏱  Timed Lock ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow)),
    )
    .alignment(Alignment::Left);

    frame.render_widget(prompt, chunks[1]);
}
