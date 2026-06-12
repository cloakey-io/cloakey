//! CloaKey Settings — configuration and user preferences.
//!
//! Provides types and utilities for loading, saving, and validating
//! CloaKey configuration from TOML files in `%APPDATA%\CloaKey\`.
//!
//! # Quick Start
//!
//! ```no_run
//! use cloakey_settings::ConfigManager;
//!
//! let manager = ConfigManager::new().expect("could not find settings directory");
//! let config = manager.load().expect("could not load config");
//!
//! println!("Overlay mode: {}", config.general.overlay);
//! println!("Emergency unlock: {}", config.shortcuts.emergency_unlock);
//! ```

pub mod config;
pub mod error;
pub mod overlay;
pub mod shortcuts;

// Re-export the most commonly used types at the crate root
pub use config::{CloaKeyConfig, ConfigManager, GeneralSettings, StartupSettings};
pub use error::SettingsError;
pub use overlay::{LogoSettings, LogoSize, OverlayMode};
pub use shortcuts::{Shortcut, ShortcutConfig};
