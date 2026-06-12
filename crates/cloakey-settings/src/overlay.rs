//! Overlay configuration types — display mode and logo settings.

use serde::{Deserialize, Serialize};

/// How the CloaKey overlay is displayed when a lock is active.
///
/// The overlay is always click-through and never intercepts input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverlayMode {
    /// No overlay displayed. Silent protection mode.
    None,
    /// Small status text in the bottom-right corner of the primary monitor.
    Minimal,
    /// Centered translucent status display with lock mode and timer.
    Standard,
    /// Reserved for a future release with larger, more prominent indicators.
    Prominent,
}

impl Default for OverlayMode {
    fn default() -> Self {
        OverlayMode::Standard
    }
}

impl std::fmt::Display for OverlayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OverlayMode::None => write!(f, "None"),
            OverlayMode::Minimal => write!(f, "Minimal"),
            OverlayMode::Standard => write!(f, "Standard"),
            OverlayMode::Prominent => write!(f, "Prominent"),
        }
    }
}

/// Logo display size inside the overlay.
///
/// Only used when `show_logo` is true and the overlay mode supports it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogoSize {
    Small,
    Medium,
    Large,
}

impl Default for LogoSize {
    fn default() -> Self {
        LogoSize::Medium
    }
}

/// Settings that control the logo appearance inside the overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoSettings {
    /// Whether to display the CloaKey logo in the overlay.
    pub show: bool,

    /// Logo opacity as a percentage (0 = fully transparent, 100 = fully opaque).
    ///
    /// Default: 60
    pub opacity: u8,

    /// Display size of the logo.
    pub size: LogoSize,
}

impl Default for LogoSettings {
    fn default() -> Self {
        Self {
            show: true,
            opacity: 60,
            size: LogoSize::default(),
        }
    }
}

impl LogoSettings {
    /// Validate logo settings, returning an error message if invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.opacity > 100 {
            return Err(format!(
                "opacity must be between 0 and 100, got {}",
                self.opacity
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_mode_default_is_standard() {
        assert_eq!(OverlayMode::default(), OverlayMode::Standard);
    }

    #[test]
    fn logo_settings_default_is_valid() {
        let settings = LogoSettings::default();
        assert!(settings.validate().is_ok());
        assert_eq!(settings.opacity, 60);
        assert!(settings.show);
    }

    #[test]
    fn logo_settings_opacity_out_of_range_fails_validation() {
        let settings = LogoSettings {
            opacity: 101,
            ..Default::default()
        };
        assert!(settings.validate().is_err());
    }

    #[test]
    fn overlay_mode_display() {
        assert_eq!(OverlayMode::None.to_string(), "None");
        assert_eq!(OverlayMode::Minimal.to_string(), "Minimal");
        assert_eq!(OverlayMode::Standard.to_string(), "Standard");
    }
}
