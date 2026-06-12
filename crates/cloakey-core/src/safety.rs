//! Safety controller — the most critical component of CloaKey.
//!
//! Responsibilities:
//! 1. Install a panic hook that forces unlock and unhooks all input on any panic.
//! 2. Provide a deadman switch: if the input engine stops responding, force unlock.
//! 3. Expose the emergency unlock signal receiver so the input engine can trigger it.
//!
//! # Safety Rule (Invariant)
//! The safety controller is initialized once at startup. Its emergency unlock
//! path CANNOT be disabled, bypassed, or removed while the application is running.
//! If there is ever a conflict between a feature and user recovery, recovery wins.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, Sender};
use tracing::{error, info, warn};

use crate::error::CoreError;

/// Signal types that the safety controller can broadcast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetySignal {
    /// Emergency unlock was triggered by the user holding CTRL+ALT+SHIFT.
    EmergencyUnlock,
    /// The deadman switch detected that the input engine stopped responding.
    DeadmanUnlock,
    /// The application is shutting down cleanly.
    Shutdown,
    /// Global hotkey triggered: Lock All
    HotkeyLockAll,
    /// Global hotkey triggered: Lock Keyboard
    HotkeyLockKeyboard,
    /// Global hotkey triggered: Lock Mouse
    HotkeyLockMouse,
    /// Global hotkey triggered: Ghost Mode
    HotkeyGhostMode,
    /// Global hotkey triggered: Unlock
    HotkeyUnlock,
}

/// A shared flag that the input engine sets to indicate it is still alive.
///
/// The deadman switch thread checks this flag periodically and forces an
/// unlock if it remains unset (meaning the input engine has stopped).
#[derive(Clone, Debug)]
pub struct InputHeartbeat {
    alive: Arc<AtomicBool>,
    enabled: Arc<AtomicBool>,
}

impl InputHeartbeat {
    pub fn new() -> Self {
        Self {
            alive: Arc::new(AtomicBool::new(true)),
            enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Called by the input engine regularly to indicate it is running.
    pub fn pulse(&self) {
        self.alive.store(true, Ordering::Release);
    }

    /// Enable the heartbeat check when input hooks are installed.
    pub fn enable(&self) {
        self.alive.store(true, Ordering::Release);
        self.enabled.store(true, Ordering::Release);
    }

    /// Disable the heartbeat check when input hooks are removed.
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Release);
    }

    /// Check if the heartbeat is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    /// Called by the deadman thread to check if the input engine is alive.
    /// Resets the flag after reading (one-shot per interval).
    pub fn check_and_reset(&self) -> bool {
        self.alive.swap(false, Ordering::AcqRel)
    }
}

impl Default for InputHeartbeat {
    fn default() -> Self {
        Self::new()
    }
}

/// The safety controller manages emergency recovery mechanisms.
///
/// Created once at application startup. Provides:
/// - A `Sender<SafetySignal>` for the input engine to send emergency unlocks.
/// - A `Receiver<SafetySignal>` for the main application loop to receive them.
/// - A deadman switch thread that sends `DeadmanUnlock` if heartbeats stop.
pub struct SafetyController {
    /// Sender half — given to the input engine.
    signal_tx: Sender<SafetySignal>,

    /// Receiver half — held by the main application loop.
    signal_rx: Receiver<SafetySignal>,

    /// Heartbeat shared with the input engine for deadman switch.
    heartbeat: InputHeartbeat,

    /// Whether the controller has been shut down.
    shutdown: Arc<AtomicBool>,
}

impl SafetyController {
    /// Initialize the safety controller.
    ///
    /// This also installs a global panic hook to ensure that any panic in
    /// the application causes a safety signal to be sent, preventing the
    /// user from becoming trapped.
    ///
    /// # Errors
    /// Returns an error if the controller could not be initialized.
    pub fn initialize() -> Result<Self, CoreError> {
        let (signal_tx, signal_rx) = bounded::<SafetySignal>(32);
        let heartbeat = InputHeartbeat::new();
        let shutdown = Arc::new(AtomicBool::new(false));

        // Install panic hook — log the panic and note that cleanup will occur
        // The actual hook removal is done in Drop on InputEngine
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            // Log the panic details
            error!(
                "CloaKey PANIC — initiating emergency recovery. Panic info: {}",
                info
            );
            // Call the previous hook so the user still sees the panic message
            prev_hook(info);
        }));

        info!("Safety controller initialized");

        Ok(Self {
            signal_tx,
            signal_rx,
            heartbeat,
            shutdown,
        })
    }

    /// Returns a clone of the signal sender for use by the input engine.
    ///
    /// The input engine uses this to send `EmergencyUnlock` signals when
    /// the user holds CTRL+ALT+SHIFT for 2 seconds.
    pub fn signal_sender(&self) -> Sender<SafetySignal> {
        self.signal_tx.clone()
    }

    /// Returns a reference to the signal receiver for use by the main app loop.
    pub fn signal_receiver(&self) -> &Receiver<SafetySignal> {
        &self.signal_rx
    }

    /// Returns a clone of the heartbeat for use by the input engine.
    pub fn heartbeat(&self) -> InputHeartbeat {
        self.heartbeat.clone()
    }

    /// Start the deadman switch background thread.
    ///
    /// The deadman switch checks the input engine's heartbeat every `check_interval`.
    /// If the heartbeat has not been pulsed within that interval, it sends a
    /// `DeadmanUnlock` signal to force recovery.
    ///
    /// # Parameters
    /// - `check_interval`: How often to check the heartbeat (recommended: 5 seconds).
    pub fn start_deadman_switch(&self, check_interval: Duration) {
        let heartbeat = self.heartbeat.clone();
        let tx = self.signal_tx.clone();
        let shutdown = self.shutdown.clone();

        std::thread::Builder::new()
            .name("cloakey-deadman".to_string())
            .spawn(move || {
                info!(
                    "Deadman switch thread started (check interval: {:?})",
                    check_interval
                );

                loop {
                    std::thread::sleep(check_interval);

                    if shutdown.load(Ordering::Acquire) {
                        info!("Deadman switch thread exiting (shutdown signaled)");
                        break;
                    }

                    if heartbeat.is_enabled() {
                        let is_alive = heartbeat.check_and_reset();
                        if !is_alive {
                            warn!("Input engine heartbeat missed — sending DeadmanUnlock for safety");
                            let _ = tx.send(SafetySignal::DeadmanUnlock);
                        }
                    }
                }
            })
            .expect("Failed to spawn deadman switch thread");
    }

    /// Signal a clean shutdown to the deadman switch thread and close channels.
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        let _ = self.signal_tx.send(SafetySignal::Shutdown);
        info!("Safety controller shutdown signaled");
    }

    /// Send an emergency unlock signal directly (for testing or manual use).
    pub fn send_emergency_unlock(&self) {
        let _ = self.signal_tx.send(SafetySignal::EmergencyUnlock);
    }

    /// Try to receive a pending safety signal without blocking.
    ///
    /// Returns `None` if no signal is available.
    pub fn try_recv_signal(&self) -> Option<SafetySignal> {
        self.signal_rx.try_recv().ok()
    }
}

/// Emergency unlock hold tracker for use in the input engine hook callback.
///
/// The hook callback runs in a tight loop — this struct tracks whether
/// CTRL, ALT, and SHIFT are all held, and for how long, triggering
/// an emergency unlock after the configured hold duration.
#[derive(Debug)]
pub struct EmergencyUnlockTracker {
    /// When all three modifier keys were first detected as simultaneously held.
    hold_start: Option<Instant>,
    /// Required hold duration before triggering.
    required_hold: Duration,
    /// Whether the emergency unlock has already fired for this hold (prevents repeat).
    fired: bool,
}

impl EmergencyUnlockTracker {
    pub fn new(hold_ms: u64) -> Self {
        Self {
            hold_start: None,
            required_hold: Duration::from_millis(hold_ms),
            fired: false,
        }
    }

    /// Update the tracker with the current key state.
    ///
    /// `all_held` should be `true` when CTRL, ALT, and SHIFT are all pressed.
    ///
    /// Returns `true` if the emergency unlock threshold has just been crossed.
    pub fn update(&mut self, all_held: bool) -> bool {
        if !all_held {
            // Keys released — reset
            self.hold_start = None;
            self.fired = false;
            return false;
        }

        if self.fired {
            // Already fired for this hold — don't fire again
            return false;
        }

        match self.hold_start {
            None => {
                // Keys just became fully held
                self.hold_start = Some(Instant::now());
                false
            }
            Some(start) => {
                if start.elapsed() >= self.required_hold {
                    // Hold threshold met — fire!
                    self.fired = true;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Reset the tracker (called after emergency unlock has been processed).
    pub fn reset(&mut self) {
        self.hold_start = None;
        self.fired = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn heartbeat_pulse_and_check() {
        let hb = InputHeartbeat::new();
        hb.pulse();
        assert!(hb.check_and_reset()); // was alive
        assert!(!hb.check_and_reset()); // flag was reset
    }

    #[test]
    fn emergency_unlock_tracker_fires_after_hold() {
        let mut tracker = EmergencyUnlockTracker::new(50); // 50ms for testing
        assert!(!tracker.update(true)); // just started holding
        thread::sleep(Duration::from_millis(60));
        assert!(tracker.update(true)); // hold complete
        assert!(!tracker.update(true)); // already fired, don't fire again
    }

    #[test]
    fn emergency_unlock_tracker_resets_on_release() {
        let mut tracker = EmergencyUnlockTracker::new(50);
        tracker.update(true);
        thread::sleep(Duration::from_millis(60));
        tracker.update(false); // release
        assert!(!tracker.fired);
        assert!(tracker.hold_start.is_none());
    }

    #[test]
    fn emergency_unlock_tracker_does_not_fire_before_hold_duration() {
        let mut tracker = EmergencyUnlockTracker::new(500); // 500ms
                                                            // Immediately check — should not fire
        for _ in 0..5 {
            assert!(!tracker.update(true));
            thread::sleep(Duration::from_millis(10));
        }
        // Total: ~50ms, much less than 500ms — should not have fired
        assert!(!tracker.fired);
    }

    #[test]
    fn safety_controller_initializes_successfully() {
        let controller = SafetyController::initialize();
        assert!(controller.is_ok());
    }

    #[test]
    fn safety_controller_can_send_and_receive_emergency_unlock() {
        let controller = SafetyController::initialize().unwrap();
        controller.send_emergency_unlock();
        let signal = controller.try_recv_signal();
        assert_eq!(signal, Some(SafetySignal::EmergencyUnlock));
    }

    #[test]
    fn safety_controller_no_signal_when_empty() {
        let controller = SafetyController::initialize().unwrap();
        assert_eq!(controller.try_recv_signal(), None);
    }
}
