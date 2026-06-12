//! Error types for the cloakey-core crate.

use thiserror::Error;

/// All errors that can occur in the CloaKey core engine.
#[derive(Debug, Error)]
pub enum CoreError {
    /// An invalid state transition was attempted.
    #[error("Invalid state transition: cannot go from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    /// A timer duration string could not be parsed.
    #[error("Invalid duration '{input}': expected a number of seconds (e.g. '60'), minutes (e.g. '5m'), or hours (e.g. '1h')")]
    InvalidDuration { input: String },

    /// A timer duration value is out of range.
    #[error("Duration '{value}' is out of range: must be between {min} and {max} seconds")]
    DurationOutOfRange { value: u64, min: u64, max: u64 },

    /// The safety controller failed to install.
    #[error("Safety controller failed to initialize: {reason}")]
    SafetyInitFailed { reason: String },

    /// A lock operation was attempted while already in a locked state.
    #[error("Already locked in mode: {current_mode}. Unlock first.")]
    AlreadyLocked { current_mode: String },

    /// An internal channel communication error occurred.
    #[error("Internal communication error: {reason}")]
    ChannelError { reason: String },
}
