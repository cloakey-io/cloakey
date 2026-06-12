//! clap command definitions for the CloaKey CLI.
//!
//! Two entry points:
//! 1. `cloak` (no subcommand) → interactive ratatui menu
//! 2. `cloak <subcommand>` → direct command mode

use clap::{Parser, Subcommand};

/// CloaKey — Protect the workflow. Don't stop the workflow.
///
/// Run without arguments to open the interactive menu.
/// Pass a subcommand to execute directly.
#[derive(Parser, Debug)]
#[command(
    name = "cloak",
    author,
    version,
    about = "CloaKey — Protect the workflow. Don't stop the workflow.",
    long_about = "CloaKey suppresses keyboard and mouse input while keeping all \
                  applications, AI agents, downloads, and processes running normally.\n\n\
                  Run without arguments to open the interactive menu.\n\
                  Emergency unlock: CTRL + ALT + SHIFT (hold 2 seconds)"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Direct command mode — bypass the interactive menu.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Lock the keyboard. Mouse remains fully active.
    #[command(alias = "lock-keyboard")]
    Keyboard,

    /// Lock the mouse. Keyboard remains fully active.
    #[command(alias = "lock-mouse")]
    Mouse,

    /// Lock both keyboard and mouse.
    #[command(aliases = ["both", "lock"])]
    Full,

    /// Ghost Mode: mouse cursor moves, but clicks and keyboard are blocked.
    Ghost,

    /// Lock for a set duration, then auto-unlock.
    ///
    /// Duration examples:
    ///   cloak timer 60     → 60 seconds
    ///   cloak timer 5m     → 5 minutes
    ///   cloak timer 1h     → 1 hour
    Timer {
        /// Duration string (e.g. "60", "5m", "1h").
        duration: String,
    },

    /// Show the current protection status.
    Status,

    /// Unlock — remove any active protection.
    Unlock,

    /// Open the settings menu.
    Settings,

    /// Configure Windows startup integration.
    Startup,

    /// Reset all settings to defaults (requires confirmation).
    Reset,

    /// Show help and all available commands.
    #[command(alias = "h")]
    Help,

    /// Show version, license, and links.
    About,
}
