//! CloaKey Overlay — transparent, always-on-top, click-through status display.
//!
//! The overlay manager controls whether and how the overlay is shown,
//! and provides a simple API for updating the displayed status text.

pub mod error;

#[cfg(target_os = "windows")]
mod window;

use cloakey_core::{LockState, SafetySignal};
use cloakey_settings::OverlayMode;
use crossbeam_channel::Sender;
pub use error::OverlayError;

/// Manages the CloaKey status overlay window.
///
/// The overlay is entirely optional — if mode is `None`, no window is created.
/// The overlay is click-through, so it never interferes with user interaction.
pub struct OverlayManager {
    mode: OverlayMode,
    active: bool,
}

impl OverlayManager {
    /// Create a new overlay manager with the given display mode.
    pub fn new(mode: OverlayMode) -> Self {
        Self {
            mode,
            active: false,
        }
    }

    /// Show the overlay with the current lock state.
    ///
    /// If mode is `None`, this is a no-op.
    pub fn show(
        &mut self,
        state: &LockState,
        timer_display: Option<&str>,
        signal_tx: Sender<SafetySignal>,
    ) -> Result<(), OverlayError> {
        if self.mode == OverlayMode::None {
            return Ok(());
        }

        let text = build_status_text(state, timer_display);
        let centered = self.mode == OverlayMode::Standard;

        #[cfg(target_os = "windows")]
        {
            if !self.active {
                unsafe {
                    window::create_overlay_window(centered, signal_tx)?;
                }
                self.active = true;
            }
            window::update_overlay_text(&text);
        }

        #[cfg(not(target_os = "windows"))]
        {
            tracing::debug!("Overlay (stub): {}", text);
            self.active = true;
        }

        Ok(())
    }

    /// Update the overlay text (e.g., timer countdown update).
    pub fn update(&self, state: &LockState, timer_display: Option<&str>) {
        if !self.active || self.mode == OverlayMode::None {
            return;
        }

        let text = build_status_text(state, timer_display);

        #[cfg(target_os = "windows")]
        window::update_overlay_text(&text);

        #[cfg(not(target_os = "windows"))]
        tracing::debug!("Overlay update (stub): {}", text);
    }

    /// Hide and destroy the overlay window.
    pub fn hide(&mut self) {
        if !self.active {
            return;
        }

        #[cfg(target_os = "windows")]
        window::destroy_overlay();

        self.active = false;
    }

    /// Returns true if the overlay is currently visible.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Drop for OverlayManager {
    fn drop(&mut self) {
        self.hide();
    }
}

/// Build the status text string for a given lock state and optional timer.
fn build_status_text(state: &LockState, timer_display: Option<&str>) -> String {
    let base = match state {
        LockState::Unlocked => return "CloaKey — Unlocked".to_string(),
        LockState::KeyboardLocked => "🔒 Keyboard Locked",
        LockState::MouseLocked => "🔒 Mouse Locked",
        LockState::FullyLocked => "🔒 Fully Locked",
        LockState::GhostMode => "👻 Ghost Mode",
        LockState::TimedLock => "⏱ Timed Lock",
    };

    if let Some(timer) = timer_display {
        format!(
            "{} — {} remaining  |  CTRL+ALT+SHIFT to unlock",
            base, timer
        )
    } else {
        format!("{}  |  CTRL+ALT+SHIFT (hold 2s) to unlock", base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_status_text_unlocked() {
        let text = build_status_text(&LockState::Unlocked, None);
        assert!(text.contains("Unlocked"));
    }

    #[test]
    fn build_status_text_keyboard_locked() {
        let text = build_status_text(&LockState::KeyboardLocked, None);
        assert!(text.contains("Keyboard Locked"));
        assert!(text.contains("CTRL+ALT+SHIFT"));
    }

    #[test]
    fn build_status_text_timed_lock_with_timer() {
        let text = build_status_text(&LockState::TimedLock, Some("4m 32s"));
        assert!(text.contains("4m 32s"));
        assert!(text.contains("remaining"));
    }

    #[test]
    fn build_status_text_ghost_mode() {
        let text = build_status_text(&LockState::GhostMode, None);
        assert!(text.contains("Ghost Mode"));
    }

    #[test]
    fn overlay_manager_none_mode_stays_inactive() {
        let mut manager = OverlayManager::new(OverlayMode::None);
        let (tx, _rx) = crossbeam_channel::bounded(1);
        // show() on None mode should not mark as active
        manager.show(&LockState::FullyLocked, None, tx).unwrap();
        assert!(!manager.is_active());
    }
}
