//! Error types for the cloakey-input crate.

use thiserror::Error;

/// All errors that can occur in the input engine.
#[derive(Debug, Error)]
pub enum InputError {
    /// A Win32 low-level hook could not be installed.
    #[error("Failed to install {hook_type} hook (Win32 error: {win32_error})")]
    HookInstallFailed {
        hook_type: &'static str,
        win32_error: u32,
    },

    /// A Win32 hook could not be removed.
    #[error("Failed to remove {hook_type} hook (Win32 error: {win32_error})")]
    HookRemoveFailed {
        hook_type: &'static str,
        win32_error: u32,
    },

    /// The hook message pump thread failed to start.
    #[error("Hook message pump thread failed to start: {reason}")]
    MessagePumpFailed { reason: String },

    /// The input engine was started while already running.
    #[error("Input engine is already running")]
    AlreadyRunning,

    /// The input engine was used after being stopped.
    #[error("Input engine has been stopped")]
    AlreadyStopped,
}
