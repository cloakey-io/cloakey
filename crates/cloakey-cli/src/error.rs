//! CLI error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Settings error: {0}")]
    Settings(#[from] cloakey_settings::SettingsError),

    #[error("Core error: {0}")]
    Core(#[from] cloakey_core::CoreError),

    #[error("Input engine error: {0}")]
    Input(#[from] cloakey_input::InputError),

    #[error("Overlay error: {0}")]
    Overlay(#[from] cloakey_overlay::OverlayError),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        CliError::Other(e.to_string())
    }
}
