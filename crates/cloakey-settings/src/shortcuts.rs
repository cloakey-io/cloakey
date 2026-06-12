//! Keyboard shortcut configuration.
//!
//! Defines all hotkeys used by CloaKey. Shortcuts are stored as human-readable
//! strings (e.g. `"ctrl+alt+l"`) and resolved to virtual key combinations at runtime.

use serde::{Deserialize, Serialize};

/// A keyboard shortcut represented as a human-readable string.
///
/// Format: modifier keys joined by `+`, then the primary key.
/// Examples: `"ctrl+alt+l"`, `"ctrl+alt+shift"`, `"ctrl+alt+k"`
///
/// Supported modifiers: `ctrl`, `alt`, `shift`, `win`
/// Keys are matched case-insensitively.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut(pub String);

impl Shortcut {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Returns the shortcut string in canonical lowercase form.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Shortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Pretty-print: "ctrl+alt+l" -> "CTRL + ALT + L"
        let parts: Vec<String> = self.0.split('+').map(|p| p.trim().to_uppercase()).collect();
        write!(f, "{}", parts.join(" + "))
    }
}

/// All configurable keyboard shortcuts for CloaKey.
///
/// Default values match the Tech Architecture specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// Lock all input (keyboard + mouse).
    /// Default: CTRL + ALT + L
    pub lock_all: Shortcut,

    /// Lock keyboard only.
    /// Default: CTRL + ALT + K
    pub lock_keyboard: Shortcut,

    /// Lock mouse only.
    /// Default: CTRL + ALT + M
    pub lock_mouse: Shortcut,

    /// Enable Ghost Mode.
    /// Default: CTRL + ALT + G
    pub ghost_mode: Shortcut,

    /// Emergency unlock — must always work.
    /// Default: CTRL + ALT + SHIFT (hold 2 seconds)
    pub emergency_unlock: Shortcut,

    /// How long to hold the emergency unlock combination (milliseconds).
    /// Default: 2000 (2 seconds)
    pub emergency_unlock_hold_ms: u64,

    /// Uncloak — unlock shortcut (immediate, no hold required).
    /// Default: CTRL + ALT + U
    pub uncloak: Shortcut,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            lock_all: Shortcut::new("ctrl+alt+l"),
            lock_keyboard: Shortcut::new("ctrl+alt+k"),
            lock_mouse: Shortcut::new("ctrl+alt+m"),
            ghost_mode: Shortcut::new("ctrl+alt+g"),
            emergency_unlock: Shortcut::new("ctrl+alt+shift"),
            emergency_unlock_hold_ms: 2000,
            uncloak: Shortcut::new("ctrl+alt+u"),
        }
    }
}

impl ShortcutConfig {
    /// Validate shortcut configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.emergency_unlock_hold_ms < 500 {
            return Err(format!(
                "emergency_unlock_hold_ms must be at least 500ms for safety, got {}",
                self.emergency_unlock_hold_ms
            ));
        }
        if self.emergency_unlock_hold_ms > 10_000 {
            return Err(format!(
                "emergency_unlock_hold_ms must not exceed 10000ms, got {}",
                self.emergency_unlock_hold_ms
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shortcuts_are_correct() {
        let config = ShortcutConfig::default();
        assert_eq!(config.lock_all.as_str(), "ctrl+alt+l");
        assert_eq!(config.lock_keyboard.as_str(), "ctrl+alt+k");
        assert_eq!(config.lock_mouse.as_str(), "ctrl+alt+m");
        assert_eq!(config.ghost_mode.as_str(), "ctrl+alt+g");
        assert_eq!(config.emergency_unlock.as_str(), "ctrl+alt+shift");
        assert_eq!(config.emergency_unlock_hold_ms, 2000);
        assert_eq!(config.uncloak.as_str(), "ctrl+alt+u");
    }

    #[test]
    fn default_shortcuts_are_valid() {
        assert!(ShortcutConfig::default().validate().is_ok());
    }

    #[test]
    fn emergency_unlock_hold_too_short_fails() {
        let config = ShortcutConfig {
            emergency_unlock_hold_ms: 100,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn emergency_unlock_hold_too_long_fails() {
        let config = ShortcutConfig {
            emergency_unlock_hold_ms: 15_000,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn shortcut_display_formats_correctly() {
        let shortcut = Shortcut::new("ctrl+alt+shift");
        assert_eq!(shortcut.to_string(), "CTRL + ALT + SHIFT");
    }
}
