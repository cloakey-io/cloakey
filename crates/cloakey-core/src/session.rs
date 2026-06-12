//! Session manager — tracks the active lock session including mode, timer, and start time.

use crate::{
    error::CoreError,
    state::{LockMode, LockState, LockTransition},
    timer::CountdownTimer,
};
use std::time::Instant;

/// The complete state of an active CloaKey session.
///
/// This is the single source of truth for what's currently locked,
/// how long it's been active, and what the timer status is.
#[derive(Debug)]
pub struct SessionState {
    /// Current lock state.
    lock_state: LockState,

    /// The current lock mode (derived from `lock_state`, cached for fast access).
    lock_mode: LockMode,

    /// When the current lock session started (for metrics/display).
    locked_at: Option<Instant>,

    /// Active countdown timer (only present in `TimedLock` state).
    timer: Option<CountdownTimer>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            lock_state: LockState::Unlocked,
            lock_mode: LockMode::for_state(&LockState::Unlocked),
            locked_at: None,
            timer: None,
        }
    }
}

impl SessionState {
    /// Create a new session starting in the `Unlocked` state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current lock state.
    pub fn lock_state(&self) -> &LockState {
        &self.lock_state
    }

    /// Returns the current lock mode (what is and isn't blocked).
    pub fn lock_mode(&self) -> &LockMode {
        &self.lock_mode
    }

    /// Returns true if protection is currently active (not in `Unlocked` state).
    pub fn is_locked(&self) -> bool {
        self.lock_state != LockState::Unlocked
    }

    /// Returns the time at which the current lock was activated, if any.
    pub fn locked_at(&self) -> Option<Instant> {
        self.locked_at
    }

    /// Returns a reference to the active countdown timer, if any.
    pub fn timer(&self) -> Option<&CountdownTimer> {
        self.timer.as_ref()
    }

    /// Attempt to transition to a new lock state.
    ///
    /// Validates the transition, updates internal state, and starts or
    /// clears any associated timer.
    ///
    /// # Errors
    /// Returns `CoreError::InvalidTransition` if the transition is not allowed.
    pub fn transition_to(&mut self, new_state: LockState) -> Result<(), CoreError> {
        LockTransition::validate(&self.lock_state, &new_state)?;

        let entering_lock = new_state != LockState::Unlocked;
        let leaving_lock = self.lock_state != LockState::Unlocked;

        // Update timer
        if entering_lock {
            self.locked_at = Some(Instant::now());
        } else {
            self.locked_at = None;
            self.timer = None;
        }

        // Clear timer if we're leaving a timed lock
        if leaving_lock && matches!(new_state, LockState::Unlocked) {
            self.timer = None;
        }

        self.lock_mode = LockMode::for_state(&new_state);
        self.lock_state = new_state;

        Ok(())
    }

    /// Start a timed lock with the given countdown timer.
    ///
    /// Transitions to `TimedLock` state and attaches the timer.
    ///
    /// # Errors
    /// Returns an error if the session is not currently `Unlocked`.
    pub fn start_timed_lock(&mut self, timer: CountdownTimer) -> Result<(), CoreError> {
        LockTransition::validate(&self.lock_state, &LockState::TimedLock)?;
        self.locked_at = Some(Instant::now());
        self.timer = Some(timer);
        self.lock_mode = LockMode::for_state(&LockState::TimedLock);
        self.lock_state = LockState::TimedLock;
        Ok(())
    }

    /// Unlock — transition back to `Unlocked` regardless of current state.
    ///
    /// This is always valid. Used by both manual unlock and emergency unlock.
    pub fn unlock(&mut self) {
        self.lock_state = LockState::Unlocked;
        self.lock_mode = LockMode::for_state(&LockState::Unlocked);
        self.locked_at = None;
        self.timer = None;
    }

    /// Check if a timed lock has expired. Returns `true` if the timer has run out.
    ///
    /// The caller is responsible for calling `unlock()` when this returns `true`.
    pub fn timer_expired(&self) -> bool {
        self.timer.as_ref().map(|t| t.is_expired()).unwrap_or(false)
    }

    /// Returns a human-readable status description for display.
    pub fn status_display(&self) -> String {
        match &self.lock_state {
            LockState::Unlocked => "Unlocked — all input is active".to_string(),
            LockState::KeyboardLocked => {
                "Keyboard Locked — keyboard blocked, mouse active".to_string()
            }
            LockState::MouseLocked => "Mouse Locked — mouse blocked, keyboard active".to_string(),
            LockState::FullyLocked => "Fully Locked — keyboard and mouse blocked".to_string(),
            LockState::GhostMode => {
                "Ghost Mode — cursor moves, clicks and keyboard blocked".to_string()
            }
            LockState::TimedLock => {
                if let Some(timer) = &self.timer {
                    format!(
                        "Timed Lock — fully locked, {} remaining",
                        timer.remaining_display()
                    )
                } else {
                    "Timed Lock — fully locked".to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn new_session_is_unlocked() {
        let session = SessionState::new();
        assert_eq!(session.lock_state(), &LockState::Unlocked);
        assert!(!session.is_locked());
        assert!(session.locked_at().is_none());
        assert!(session.timer().is_none());
    }

    #[test]
    fn transition_to_keyboard_locked() {
        let mut session = SessionState::new();
        session.transition_to(LockState::KeyboardLocked).unwrap();
        assert_eq!(session.lock_state(), &LockState::KeyboardLocked);
        assert!(session.is_locked());
        assert!(session.locked_at().is_some());
    }

    #[test]
    fn unlock_always_succeeds() {
        let mut session = SessionState::new();
        session.transition_to(LockState::FullyLocked).unwrap();
        assert!(session.is_locked());
        session.unlock();
        assert!(!session.is_locked());
        assert_eq!(session.lock_state(), &LockState::Unlocked);
    }

    #[test]
    fn locked_to_locked_transition_fails() {
        let mut session = SessionState::new();
        session.transition_to(LockState::KeyboardLocked).unwrap();
        let result = session.transition_to(LockState::MouseLocked);
        assert!(result.is_err());
        // State must remain unchanged
        assert_eq!(session.lock_state(), &LockState::KeyboardLocked);
    }

    #[test]
    fn timed_lock_stores_timer() {
        let mut session = SessionState::new();
        let timer = CountdownTimer::new(Duration::from_secs(60));
        session.start_timed_lock(timer).unwrap();
        assert_eq!(session.lock_state(), &LockState::TimedLock);
        assert!(session.timer().is_some());
    }

    #[test]
    fn timed_lock_timer_expiry_detected() {
        let mut session = SessionState::new();
        let timer = CountdownTimer::new(Duration::from_millis(1));
        session.start_timed_lock(timer).unwrap();
        std::thread::sleep(Duration::from_millis(10));
        assert!(session.timer_expired());
    }

    #[test]
    fn unlock_clears_timer() {
        let mut session = SessionState::new();
        let timer = CountdownTimer::new(Duration::from_secs(60));
        session.start_timed_lock(timer).unwrap();
        session.unlock();
        assert!(session.timer().is_none());
    }

    #[test]
    fn all_lock_modes_can_be_entered_from_unlocked() {
        let modes = [
            LockState::KeyboardLocked,
            LockState::MouseLocked,
            LockState::FullyLocked,
            LockState::GhostMode,
        ];
        for mode in modes {
            let mut session = SessionState::new();
            session.transition_to(mode.clone()).unwrap();
            assert_eq!(session.lock_state(), &mode);
            session.unlock();
        }
    }
}
