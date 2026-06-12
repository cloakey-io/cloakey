//! CloaKey Input Engine — Windows low-level keyboard and mouse hooks.
//!
//! # Overview
//! The input engine installs two global Win32 hooks:
//! - `WH_KEYBOARD_LL` — intercepts all keyboard events system-wide
//! - `WH_MOUSE_LL` — intercepts all mouse events system-wide
//!
//! These hooks run on a dedicated background thread with a Win32 message pump.
//! The main thread communicates with the hook thread via atomic shared state.
//!
//! # Safety Guarantee
//! The `InputEngine` implements `Drop` to guarantee hook removal even if the
//! application panics or the engine is dropped unexpectedly. This prevents the
//! user from becoming trapped with blocked input.
//!
//! # Privacy
//! The input engine ONLY blocks events. It NEVER:
//! - Records keystrokes
//! - Saves key presses or passwords
//! - Transmits input data
//! - Stores clipboard contents

#[cfg(target_os = "windows")]
pub mod emergency;
#[cfg(target_os = "windows")]
mod hooks;
#[cfg(target_os = "windows")]
mod keyboard;
#[cfg(target_os = "windows")]
mod mouse;

pub mod error;

pub use error::InputError;

#[cfg(target_os = "windows")]
pub use engine::InputEngine;

#[cfg(target_os = "windows")]
mod engine {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::thread::{self, JoinHandle};

    use crossbeam_channel::Sender;
    use tracing::{debug, error, info, warn};
    use windows::Win32::UI::WindowsAndMessaging::HHOOK;

    use cloakey_core::{InputHeartbeat, LockMode, SafetySignal, EmergencyUnlockTracker};

    use crate::{
        error::InputError,
        emergency::all_emergency_keys_held,
        hooks::{
            install_keyboard_hook, install_mouse_hook, remove_hook, run_message_pump,
            stop_message_pump,
        },
        keyboard::{init_keyboard_state, keyboard_hook_callback, update_lock_mode},
        mouse::{init_mouse_state, mouse_hook_callback, update_mouse_lock_mode},
    };

    /// Handle to the hook thread, used to signal it to stop.
    static mut KB_HOOK_HANDLE: HHOOK = HHOOK(0 as _);
    static mut MOUSE_HOOK_HANDLE: HHOOK = HHOOK(0 as _);

    /// The main input engine — manages the lifecycle of Win32 global input hooks.
    ///
    /// Creating an `InputEngine` installs the hooks. Dropping it removes them.
    /// The engine runs a dedicated background thread with a Win32 message pump.
    pub struct InputEngine {
        /// Background thread running the Win32 message pump.
        hook_thread: Option<JoinHandle<()>>,
        /// Signals the hook thread to stop.
        stop_flag: Arc<AtomicBool>,
        /// Heartbeat for the deadman switch (cloned into hook thread).
        #[allow(dead_code)]
        heartbeat: InputHeartbeat,
        /// Whether the engine is currently active.
        running: Arc<AtomicBool>,
    }

    impl InputEngine {
        /// Create and start the input engine.
        ///
        /// Installs keyboard and mouse hooks on a dedicated background thread.
        /// The hooks are active as soon as this returns `Ok(engine)`.
        ///
        /// # Parameters
        /// - `initial_mode`: The initial lock mode (what to block from the start).
        /// - `unlock_hold_ms`: How long CTRL+ALT+SHIFT must be held for emergency unlock.
        /// - `signal_tx`: Channel to send safety signals to the main application.
        /// - `heartbeat`: Heartbeat shared with the safety controller's deadman switch.
        ///
        /// # Errors
        /// Returns `InputError::HookInstallFailed` if a hook could not be installed.
        /// In this case, no hooks remain installed (cleanup is performed automatically).
        pub fn start(
            initial_mode: LockMode,
            unlock_hold_ms: u64,
            signal_tx: Sender<SafetySignal>,
            heartbeat: InputHeartbeat,
        ) -> Result<Self, InputError> {
            let stop_flag = Arc::new(AtomicBool::new(false));
            let running = Arc::new(AtomicBool::new(false));

            // Enable heartbeat checks
            heartbeat.enable();

            let stop_flag_clone = stop_flag.clone();
            let running_clone = running.clone();
            let initial_mode_clone = initial_mode.clone();
            let signal_tx_clone = signal_tx.clone();
            let heartbeat_clone = heartbeat.clone();

            // The hook thread channel — used to receive the hook handles back
            // so we can store them for later removal.
            let (ready_tx, ready_rx) = crossbeam_channel::bounded::<Result<(), InputError>>(1);

            // Spawn the hook thread — hooks MUST be installed and pumped from the same thread
            let hook_thread = thread::Builder::new()
                .name("cloakey-input-hooks".to_string())
                .spawn(move || {
                    // Initialize keyboard and mouse global state
                    init_keyboard_state(
                        initial_mode_clone.clone(),
                        signal_tx_clone.clone(),
                    );
                    init_mouse_state(initial_mode_clone);

                    // Install hooks
                    let result = unsafe {
                        let kb = install_keyboard_hook(Some(keyboard_hook_callback));
                        match kb {
                            Err(e) => Err(e),
                            Ok(kb_hook) => {
                                let ms = install_mouse_hook(Some(mouse_hook_callback));
                                match ms {
                                    Err(e) => {
                                        // Keyboard hook installed but mouse failed — clean up keyboard
                                        remove_hook(kb_hook, "WH_KEYBOARD_LL");
                                        Err(e)
                                    }
                                    Ok(ms_hook) => {
                                        // Store handles in statics for Drop cleanup
                                        KB_HOOK_HANDLE = kb_hook;
                                        MOUSE_HOOK_HANDLE = ms_hook;
                                        Ok(())
                                    }
                                }
                            }
                        }
                    };

                    // Signal the main thread whether hook installation succeeded
                    let _ = ready_tx.send(result);

                    running_clone.store(true, Ordering::Release);

                    // Initialize the emergency hold tracker (hold 2 seconds to unlock)
                    let mut emergency_tracker = EmergencyUnlockTracker::new(unlock_hold_ms);

                    // Run the message pump loop (non-blocking, with heartbeat pulsing)
                    loop {
                        if stop_flag_clone.load(Ordering::Acquire) {
                            break;
                        }

                        // Pulse heartbeat so the deadman switch knows we're alive
                        heartbeat_clone.pulse();

                        // Check physical emergency keys hold state (bypasses Windows message repeat limits)
                        let all_held = all_emergency_keys_held();
                        if emergency_tracker.update(all_held) {
                            warn!("Emergency hold detected on input thread — triggering unlock");
                            let _ = signal_tx_clone.send(SafetySignal::EmergencyUnlock);
                        }

                        // Process all pending Win32 messages (non-blocking)
                        let quit = unsafe { run_message_pump() };
                        if quit {
                            break;
                        }

                        // Sleep briefly to avoid busy-spinning (10ms tick)
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }

                    running_clone.store(false, Ordering::Release);
                    info!("Input hook thread exiting");
                })
                .map_err(|e| InputError::MessagePumpFailed {
                    reason: e.to_string(),
                })?;

            // Wait for hook installation to complete
            match ready_rx.recv() {
                Ok(Ok(())) => {
                    info!("Input engine started successfully");
                    Ok(Self {
                        hook_thread: Some(hook_thread),
                        stop_flag,
                        heartbeat,
                        running,
                    })
                }
                Ok(Err(e)) => {
                    error!("Hook installation failed: {}", e);
                    Err(e)
                }
                Err(_) => Err(InputError::MessagePumpFailed {
                    reason: "Hook thread exited before signaling readiness".to_string(),
                }),
            }
        }

        /// Update the lock mode — called when the lock state changes.
        ///
        /// Thread-safe. Can be called from any thread.
        pub fn set_lock_mode(&self, mode: LockMode) {
            update_lock_mode(mode.clone());
            update_mouse_lock_mode(mode);
            debug!("Input engine lock mode updated");
        }

        /// Returns true if the engine's hook thread is running.
        pub fn is_running(&self) -> bool {
            self.running.load(Ordering::Acquire)
        }

        /// Stop the input engine and remove all hooks.
        ///
        /// This is called automatically by `Drop`, but can be called explicitly
        /// for a graceful controlled shutdown.
        pub fn stop(&mut self) {
            if self.stop_flag.swap(true, Ordering::AcqRel) {
                return; // Already stopped
            }

            info!("Stopping input engine");
            self.heartbeat.disable();

            unsafe {
                // Remove hooks first
                remove_hook(KB_HOOK_HANDLE, "WH_KEYBOARD_LL");
                remove_hook(MOUSE_HOOK_HANDLE, "WH_MOUSE_LL");
                KB_HOOK_HANDLE = HHOOK(0 as _);
                MOUSE_HOOK_HANDLE = HHOOK(0 as _);

                // Signal the message pump to exit
                stop_message_pump();
            }

            // Wait for the hook thread to finish
            if let Some(thread) = self.hook_thread.take() {
                if thread.join().is_err() {
                    warn!("Hook thread did not exit cleanly");
                }
            }

            self.running.store(false, Ordering::Release);
            info!("Input engine stopped");
        }
    }

    /// `Drop` implementation guarantees hook removal even if the engine is dropped
    /// unexpectedly (panic, early return, etc.).
    ///
    /// This is critical for user safety — we must NEVER leave input hooks installed
    /// when CloaKey is no longer running.
    impl Drop for InputEngine {
        fn drop(&mut self) {
            self.stop();
        }
    }
}

/// Stub engine for non-Windows builds (enables compilation for testing on Linux/Mac CI).
#[cfg(not(target_os = "windows"))]
pub struct InputEngine;

#[cfg(not(target_os = "windows"))]
impl InputEngine {
    pub fn start(
        _mode: cloakey_core::LockMode,
        _hold_ms: u64,
        _signal_tx: crossbeam_channel::Sender<cloakey_core::SafetySignal>,
        _heartbeat: cloakey_core::InputHeartbeat,
    ) -> Result<Self, InputError> {
        tracing::warn!("InputEngine: Not running on Windows — hooks are no-ops");
        Ok(Self)
    }

    pub fn set_lock_mode(&self, _mode: cloakey_core::LockMode) {}
    pub fn is_running(&self) -> bool {
        false
    }
    pub fn stop(&mut self) {}
}
