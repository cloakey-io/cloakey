//! Configuration loading, saving, and validation.
//!
//! Settings are stored as TOML files in `%APPDATA%\CloaKey\`.
//!
//! File layout:
//! ```text
//! %APPDATA%\CloaKey\
//!   config.toml      ← general settings + overlay + logo
//!   shortcuts.toml   ← keyboard shortcut bindings
//! ```

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::{
    error::SettingsError,
    overlay::{LogoSettings, OverlayMode},
    shortcuts::ShortcutConfig,
};

/// The application name used for the config directory.
const APP_NAME: &str = "CloaKey";

/// General settings that apply to the overall CloaKey application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    /// The overlay mode to use when protection is active.
    pub overlay: OverlayMode,

    /// Logo display settings inside the overlay.
    pub logo: LogoSettings,

    /// Whether to start CloaKey minimized (in background) at launch.
    pub start_minimized: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            overlay: OverlayMode::default(),
            logo: LogoSettings::default(),
            start_minimized: false,
        }
    }
}

/// Startup integration settings (Windows Registry).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupSettings {
    /// Whether CloaKey should run automatically at Windows startup.
    pub run_on_startup: bool,

    /// Whether to start minimized when launched from startup.
    pub start_minimized: bool,
}

impl Default for StartupSettings {
    fn default() -> Self {
        Self {
            run_on_startup: false,
            start_minimized: false,
        }
    }
}

/// Root configuration structure for CloaKey.
///
/// This is serialized to and deserialized from `%APPDATA%\CloaKey\config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloaKeyConfig {
    /// General application settings.
    #[serde(default)]
    pub general: GeneralSettings,

    /// Keyboard shortcut bindings.
    #[serde(default)]
    pub shortcuts: ShortcutConfig,

    /// Windows startup integration settings.
    #[serde(default)]
    pub startup: StartupSettings,
}

impl Default for CloaKeyConfig {
    fn default() -> Self {
        Self {
            general: GeneralSettings::default(),
            shortcuts: ShortcutConfig::default(),
            startup: StartupSettings::default(),
        }
    }
}

impl CloaKeyConfig {
    /// Validate all configuration values.
    ///
    /// Returns an error if any value is out of range or invalid.
    pub fn validate(&self) -> Result<(), SettingsError> {
        // Validate logo settings
        self.general
            .logo
            .validate()
            .map_err(|reason| SettingsError::ValidationFailed {
                field: "general.logo.opacity".to_string(),
                reason,
            })?;

        // Validate shortcuts
        self.shortcuts
            .validate()
            .map_err(|reason| SettingsError::ValidationFailed {
                field: "shortcuts.emergency_unlock_hold_ms".to_string(),
                reason,
            })?;

        Ok(())
    }
}

/// Manages loading and saving CloaKey configuration files.
#[derive(Debug)]
pub struct ConfigManager {
    /// Path to the `%APPDATA%\CloaKey\` directory.
    config_dir: PathBuf,
}

impl ConfigManager {
    /// Create a new `ConfigManager`, resolving the settings directory.
    ///
    /// Returns an error if `%APPDATA%` cannot be determined.
    pub fn new() -> Result<Self, SettingsError> {
        let config_dir = dirs::config_dir()
            .map(|base| base.join(APP_NAME))
            .ok_or(SettingsError::NoSettingsDirectory)?;

        debug!("Settings directory: {}", config_dir.display());

        Ok(Self { config_dir })
    }

    /// Returns the path to the main config file.
    pub fn config_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    /// Ensure the settings directory exists, creating it if necessary.
    fn ensure_dir(&self) -> Result<(), SettingsError> {
        if !self.config_dir.exists() {
            std::fs::create_dir_all(&self.config_dir).map_err(|e| SettingsError::WriteFailed {
                path: self.config_dir.display().to_string(),
                source: e,
            })?;
            info!("Created settings directory: {}", self.config_dir.display());
        }
        Ok(())
    }

    /// Load configuration from disk.
    ///
    /// If the config file does not exist, returns default configuration.
    /// If the config file is corrupted, logs a warning and returns defaults.
    pub fn load(&self) -> Result<CloaKeyConfig, SettingsError> {
        let path = self.config_path();

        if !path.exists() {
            info!("No config file found at '{}', using defaults", path.display());
            return Ok(CloaKeyConfig::default());
        }

        let contents = std::fs::read_to_string(&path).map_err(|e| SettingsError::ReadFailed {
            path: path.display().to_string(),
            source: e,
        })?;

        match toml::from_str::<CloaKeyConfig>(&contents) {
            Ok(config) => {
                debug!("Loaded config from '{}'", path.display());
                // Validate on load; if invalid, fall back to defaults with a warning
                if let Err(e) = config.validate() {
                    warn!("Config file has invalid values ({}), using defaults", e);
                    return Ok(CloaKeyConfig::default());
                }
                Ok(config)
            }
            Err(e) => {
                warn!(
                    "Config file '{}' is corrupted ({}), using defaults",
                    path.display(),
                    e
                );
                // Don't propagate parse errors — always return usable defaults
                Ok(CloaKeyConfig::default())
            }
        }
    }

    /// Save configuration to disk.
    ///
    /// Creates the settings directory if it does not exist.
    pub fn save(&self, config: &CloaKeyConfig) -> Result<(), SettingsError> {
        config.validate()?;
        self.ensure_dir()?;

        let path = self.config_path();
        let contents = toml::to_string_pretty(config).expect("CloaKeyConfig is always serializable");

        std::fs::write(&path, contents).map_err(|e| SettingsError::WriteFailed {
            path: path.display().to_string(),
            source: e,
        })?;

        info!("Saved config to '{}'", path.display());
        Ok(())
    }

    /// Reset configuration to defaults and save.
    pub fn reset(&self) -> Result<CloaKeyConfig, SettingsError> {
        let default_config = CloaKeyConfig::default();
        self.save(&default_config)?;
        info!("Reset config to defaults");
        Ok(default_config)
    }

    /// Returns the path to the settings directory.
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper: create a ConfigManager pointing at a temporary directory.
    fn temp_manager() -> (ConfigManager, TempDir) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let manager = ConfigManager {
            config_dir: dir.path().to_path_buf(),
        };
        (manager, dir)
    }

    #[test]
    fn default_config_is_valid() {
        assert!(CloaKeyConfig::default().validate().is_ok());
    }

    #[test]
    fn load_returns_defaults_when_no_file() {
        let (manager, _dir) = temp_manager();
        let config = manager.load().expect("load should succeed");
        // Should equal defaults
        assert_eq!(config.general.overlay, OverlayMode::Standard);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (manager, _dir) = temp_manager();
        let mut config = CloaKeyConfig::default();
        config.general.overlay = OverlayMode::Minimal;
        config.general.logo.opacity = 80;

        manager.save(&config).expect("save should succeed");
        let loaded = manager.load().expect("load should succeed");

        assert_eq!(loaded.general.overlay, OverlayMode::Minimal);
        assert_eq!(loaded.general.logo.opacity, 80);
    }

    #[test]
    fn load_with_corrupt_file_returns_defaults() {
        let (manager, _dir) = temp_manager();
        fs::create_dir_all(manager.config_dir()).unwrap();
        fs::write(manager.config_path(), b"this is not valid toml !!!").unwrap();

        // Should not error — falls back to defaults gracefully
        let config = manager.load().expect("should gracefully return defaults");
        assert_eq!(config.general.overlay, OverlayMode::Standard);
    }

    #[test]
    fn reset_saves_defaults() {
        let (manager, _dir) = temp_manager();

        // First save a non-default config
        let mut config = CloaKeyConfig::default();
        config.general.overlay = OverlayMode::None;
        manager.save(&config).unwrap();

        // Reset
        manager.reset().unwrap();
        let loaded = manager.load().unwrap();
        assert_eq!(loaded.general.overlay, OverlayMode::Standard);
    }
}
