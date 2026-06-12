//! Error types for the cloakey-settings crate.

use thiserror::Error;

/// All errors that can occur when loading, saving, or validating CloaKey settings.
#[derive(Debug, Error)]
pub enum SettingsError {
    /// The settings directory could not be determined (e.g., no %APPDATA%).
    #[error("Could not determine settings directory")]
    NoSettingsDirectory,

    /// A config file could not be read from disk.
    #[error("Failed to read config file '{path}': {source}")]
    ReadFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// A config file could not be written to disk.
    #[error("Failed to write config file '{path}': {source}")]
    WriteFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// A config file contained invalid TOML.
    #[error("Failed to parse config file '{path}': {source}")]
    ParseFailed {
        path: String,
        #[source]
        source: toml::de::Error,
    },

    /// A config value failed validation (e.g., opacity out of range).
    #[error("Invalid setting '{field}': {reason}")]
    ValidationFailed { field: String, reason: String },
}
