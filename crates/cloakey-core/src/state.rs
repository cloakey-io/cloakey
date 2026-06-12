//! Lock state machine — defines all protection modes and what each blocks.
//!
//! The state machine is the central concept of CloaKey. Every transition is
//! explicit and validated. The only implicit transition is from any state
//! to `Unlocked` via the emergency unlock mechanism.

use crate::error::CoreError;
use std::fmt;

/// The current protection state of CloaKey.
///
/// Each variant describes a distinct set of blocked/allowed input.
/// Transitions are validated — only legal transitions are allowed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LockState {
    /// No protection active. All input passes through normally.
    Unlocked,

    /// Keyboard input is blocked. Mouse input is fully allowed.
    KeyboardLocked,

    /// Mouse input (movement, clicks, scroll) is blocked. Keyboard is fully allowed.
    MouseLocked,

    /// Both keyboard and mouse are fully blocked.
    FullyLocked,

    /// Ghost Mode: mouse cursor moves freely, but all clicks and keyboard input are blocked.
    GhostMode,

    /// Timed lock: same blocking as `FullyLocked`, but auto-unlocks after a timer expires.
    TimedLock,
}

impl fmt::Display for LockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockState::Unlocked => write!(f, "Unlocked"),
            LockState::KeyboardLocked => write!(f, "Keyboard Locked"),
            LockState::MouseLocked => write!(f, "Mouse Locked"),
            LockState::FullyLocked => write!(f, "Fully Locked"),
            LockState::GhostMode => write!(f, "Ghost Mode"),
            LockState::TimedLock => write!(f, "Timed Lock"),
        }
    }
}

/// Describes exactly what input is blocked in a given `LockState`.
///
/// This is used by the input engine to make per-event decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockMode {
    /// Whether keyboard key presses are blocked.
    pub keyboard_blocked: bool,
    /// Whether mouse movement is blocked.
    pub mouse_movement_blocked: bool,
    /// Whether mouse button clicks are blocked.
    pub mouse_clicks_blocked: bool,
    /// Whether mouse scroll wheel is blocked.
    pub mouse_scroll_blocked: bool,
}

impl LockMode {
    /// Returns the `LockMode` that corresponds to a given `LockState`.
    pub fn for_state(state: &LockState) -> Self {
        match state {
            LockState::Unlocked => Self {
                keyboard_blocked: false,
                mouse_movement_blocked: false,
                mouse_clicks_blocked: false,
                mouse_scroll_blocked: false,
            },
            LockState::KeyboardLocked => Self {
                keyboard_blocked: true,
                mouse_movement_blocked: false,
                mouse_clicks_blocked: false,
                mouse_scroll_blocked: false,
            },
            LockState::MouseLocked => Self {
                keyboard_blocked: false,
                mouse_movement_blocked: true,
                mouse_clicks_blocked: true,
                mouse_scroll_blocked: true,
            },
            LockState::FullyLocked | LockState::TimedLock => Self {
                keyboard_blocked: true,
                mouse_movement_blocked: true,
                mouse_clicks_blocked: true,
                mouse_scroll_blocked: true,
            },
            LockState::GhostMode => Self {
                keyboard_blocked: true,
                mouse_movement_blocked: false,
                mouse_clicks_blocked: true,
                mouse_scroll_blocked: true,
            },
        }
    }

    /// Returns true if this mode blocks any input at all.
    pub fn is_any_blocked(&self) -> bool {
        self.keyboard_blocked
            || self.mouse_movement_blocked
            || self.mouse_clicks_blocked
            || self.mouse_scroll_blocked
    }
}

/// Validates and applies lock state transitions.
///
/// Rules:
/// - Any locked state can transition to `Unlocked` (emergency unlock path).
/// - `Unlocked` can transition to any locked state.
/// - Locked-to-locked transitions are NOT allowed (must unlock first).
/// - TimedLock can self-transition to Unlocked when the timer expires.
pub struct LockTransition;

impl LockTransition {
    /// Attempt to transition from `current` to `next`.
    ///
    /// Returns `Ok(())` if the transition is valid, or a `CoreError` if not.
    pub fn validate(current: &LockState, next: &LockState) -> Result<(), CoreError> {
        match (current, next) {
            // From Unlocked: can go to any protected state
            (LockState::Unlocked, _) => Ok(()),

            // From any locked state: can always go to Unlocked (emergency unlock)
            (_, LockState::Unlocked) => Ok(()),

            // Locked-to-locked: not allowed
            (from, to) => Err(CoreError::InvalidTransition {
                from: from.to_string(),
                to: to.to_string(),
            }),
        }
    }

    /// Returns true if the transition is valid without consuming an error.
    pub fn is_valid(current: &LockState, next: &LockState) -> bool {
        Self::validate(current, next).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlocked_can_transition_to_all_states() {
        let targets = [
            LockState::KeyboardLocked,
            LockState::MouseLocked,
            LockState::FullyLocked,
            LockState::GhostMode,
            LockState::TimedLock,
        ];
        for target in &targets {
            assert!(
                LockTransition::is_valid(&LockState::Unlocked, target),
                "Expected transition to {} to be valid",
                target
            );
        }
    }

    #[test]
    fn any_locked_state_can_go_to_unlocked() {
        let sources = [
            LockState::KeyboardLocked,
            LockState::MouseLocked,
            LockState::FullyLocked,
            LockState::GhostMode,
            LockState::TimedLock,
        ];
        for source in &sources {
            assert!(
                LockTransition::is_valid(source, &LockState::Unlocked),
                "Expected transition from {} to Unlocked to be valid",
                source
            );
        }
    }

    #[test]
    fn locked_to_locked_transitions_are_rejected() {
        let states = [
            LockState::KeyboardLocked,
            LockState::MouseLocked,
            LockState::FullyLocked,
            LockState::GhostMode,
        ];
        for from in &states {
            for to in &states {
                if from == to {
                    continue;
                }
                assert!(
                    !LockTransition::is_valid(from, to),
                    "Expected transition from {} to {} to be invalid",
                    from,
                    to
                );
            }
        }
    }

    #[test]
    fn lock_mode_for_unlocked_blocks_nothing() {
        let mode = LockMode::for_state(&LockState::Unlocked);
        assert!(!mode.is_any_blocked());
    }

    #[test]
    fn lock_mode_for_keyboard_locked_blocks_only_keyboard() {
        let mode = LockMode::for_state(&LockState::KeyboardLocked);
        assert!(mode.keyboard_blocked);
        assert!(!mode.mouse_movement_blocked);
        assert!(!mode.mouse_clicks_blocked);
        assert!(!mode.mouse_scroll_blocked);
    }

    #[test]
    fn lock_mode_for_ghost_mode_allows_mouse_movement() {
        let mode = LockMode::for_state(&LockState::GhostMode);
        assert!(mode.keyboard_blocked);
        assert!(!mode.mouse_movement_blocked);
        assert!(mode.mouse_clicks_blocked);
        assert!(mode.mouse_scroll_blocked);
    }

    #[test]
    fn lock_mode_for_fully_locked_blocks_everything() {
        let mode = LockMode::for_state(&LockState::FullyLocked);
        assert!(mode.keyboard_blocked);
        assert!(mode.mouse_movement_blocked);
        assert!(mode.mouse_clicks_blocked);
        assert!(mode.mouse_scroll_blocked);
        assert!(mode.is_any_blocked());
    }

    #[test]
    fn lock_state_display_names() {
        assert_eq!(LockState::Unlocked.to_string(), "Unlocked");
        assert_eq!(LockState::KeyboardLocked.to_string(), "Keyboard Locked");
        assert_eq!(LockState::GhostMode.to_string(), "Ghost Mode");
        assert_eq!(LockState::TimedLock.to_string(), "Timed Lock");
    }
}
