//! Error types for the overlay system.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OverlayError {
    /// The overlay window could not be created.
    #[error("Failed to create overlay window (Win32 error: {win32_error})")]
    WindowCreateFailed { win32_error: u32 },

    /// The overlay was used after being destroyed.
    #[error("Overlay has already been destroyed")]
    AlreadyDestroyed,

    /// Failed to register the window class.
    #[error("Failed to register overlay window class (Win32 error: {win32_error})")]
    ClassRegisterFailed { win32_error: u32 },
}
