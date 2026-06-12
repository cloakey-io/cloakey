//! CloaKey Core — lock state machine, timer engine, session manager, safety controller.
//!
//! This crate contains all business logic for CloaKey with zero OS-specific code.
//! Platform adapters live in `cloakey-input` and `cloakey-overlay`.
//!
//! # Key Types
//! - [`state::LockState`] — the 6 protection modes
//! - [`state::LockMode`] — what each mode blocks
//! - [`session::SessionState`] — active session tracking
//! - [`timer::CountdownTimer`] — timed lock countdown
//! - [`timer::parse_duration`] — parse "60", "5m", "1h" strings
//! - [`safety::SafetyController`] — emergency recovery, deadman switch, panic hook
//! - [`safety::EmergencyUnlockTracker`] — CTRL+ALT+SHIFT hold detection

pub mod error;
pub mod safety;
pub mod session;
pub mod state;
pub mod timer;

// Convenient re-exports
pub use error::CoreError;
pub use safety::{EmergencyUnlockTracker, InputHeartbeat, SafetyController, SafetySignal};
pub use session::SessionState;
pub use state::{LockMode, LockState, LockTransition};
pub use timer::{parse_duration, CountdownTimer};
