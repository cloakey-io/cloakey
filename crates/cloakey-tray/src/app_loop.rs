use std::time::Duration;
use cloakey_core::{LockState, SafetyController, SafetySignal, SessionState};
use cloakey_settings::{CloaKeyConfig, ConfigManager};
use cloakey_input::InputEngine;
use cloakey_overlay::OverlayManager;

use crate::tray::TrayManager;
use crate::hotkeys::HotkeyManager;

/// Start the background tray daemon event loop.
pub fn run_daemon() -> Result<(), String> {
    // 1. Load configuration
    let config_manager = ConfigManager::new().map_err(|e| e.to_string())?;
    let config = config_manager.load().map_err(|e| e.to_string())?;
    
    // 2. Initialize Safety Controller
    let safety = SafetyController::initialize().map_err(|e| e.to_string())?;
    safety.start_deadman_switch(Duration::from_secs(5));
    
    // 3. Initialize state and overlay managers
    let mut session = SessionState::new();
    let mut input_engine: Option<InputEngine> = None;
    let mut overlay = OverlayManager::new(config.general.overlay.clone());
    
    // 4. Initialize System Tray and Global Hotkeys
    let tray = TrayManager::new()?;
    let hotkeys = HotkeyManager::new()?;
    
    // Set initial tray state
    tray.update_state(&LockState::Unlocked);
    
    let mut running = true;
    
    use windows::Win32::UI::WindowsAndMessaging::{
        PeekMessageW, TranslateMessage, DispatchMessageW, PM_REMOVE, MSG
    };
    
    tracing::info!("CloaKey background daemon loop started");
    
    while running {
        // --- Win32 Message Pump (Required for tray clicks & hotkey events) ---
        let mut msg = MSG::default();
        unsafe {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }
        }
        
        // --- Process Menu Events ---
        while let Ok(event) = muda::MenuEvent::receiver().try_recv() {
            if let Some(signal) = tray.handle_menu_event(&event.id) {
                process_signal(
                    signal,
                    &mut running,
                    &mut session,
                    &mut input_engine,
                    &mut overlay,
                    &tray,
                    &safety,
                    &config,
                )?;
            }
        }
        
        // --- Process Hotkey Events ---
        while let Ok(event) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
            if let Some(signal) = hotkeys.handle_hotkey_event(event.id) {
                process_signal(
                    signal,
                    &mut running,
                    &mut session,
                    &mut input_engine,
                    &mut overlay,
                    &tray,
                    &safety,
                    &config,
                )?;
            }
        }
        
        // --- Process Safety Signals (Emergency Unlock / Deadman) ---
        while let Some(signal) = safety.try_recv_signal() {
            process_signal(
                signal,
                &mut running,
                &mut session,
                &mut input_engine,
                &mut overlay,
                &tray,
                &safety,
                &config,
            )?;
        }
        
        // --- Timed Lock Countdown ---
        if let Some(timer) = session.timer() {
            if session.timer_expired() {
                perform_unlock(&mut session, &mut input_engine, &mut overlay, &tray)?;
            } else {
                overlay.update(session.lock_state(), Some(&timer.remaining_display()));
            }
        }
        
        std::thread::sleep(Duration::from_millis(50));
    }
    
    // Cleanup active locks on exit
    let _ = perform_unlock(&mut session, &mut input_engine, &mut overlay, &tray);
    safety.shutdown();
    
    tracing::info!("CloaKey background daemon loop stopped");
    Ok(())
}

fn process_signal(
    signal: SafetySignal,
    running: &mut bool,
    session: &mut SessionState,
    input_engine: &mut Option<InputEngine>,
    overlay: &mut OverlayManager,
    tray: &TrayManager,
    safety: &SafetyController,
    config: &CloaKeyConfig,
) -> Result<(), String> {
    match signal {
        SafetySignal::HotkeyLockAll => {
            perform_lock(LockState::FullyLocked, session, input_engine, overlay, tray, safety, config)?;
        }
        SafetySignal::HotkeyLockKeyboard => {
            perform_lock(LockState::KeyboardLocked, session, input_engine, overlay, tray, safety, config)?;
        }
        SafetySignal::HotkeyLockMouse => {
            perform_lock(LockState::MouseLocked, session, input_engine, overlay, tray, safety, config)?;
        }
        SafetySignal::HotkeyGhostMode => {
            perform_lock(LockState::GhostMode, session, input_engine, overlay, tray, safety, config)?;
        }
        SafetySignal::EmergencyUnlock | SafetySignal::DeadmanUnlock | SafetySignal::HotkeyUnlock => {
            perform_unlock(session, input_engine, overlay, tray)?;
        }
        SafetySignal::Shutdown => {
            *running = false;
        }
    }
    Ok(())
}

fn perform_lock(
    state: LockState,
    session: &mut SessionState,
    input_engine: &mut Option<InputEngine>,
    overlay: &mut OverlayManager,
    tray: &TrayManager,
    safety: &SafetyController,
    config: &CloaKeyConfig,
) -> Result<(), String> {
    if *session.lock_state() != LockState::Unlocked {
        return Ok(()); // Already locked
    }
    
    session.transition_to(state.clone()).map_err(|e| e.to_string())?;
    
    let mode = session.lock_mode().clone();
    let hold_ms = config.shortcuts.emergency_unlock_hold_ms;
    let signal_tx = safety.signal_sender();
    let heartbeat = safety.heartbeat();
    
    let engine = InputEngine::start(mode, hold_ms, signal_tx.clone(), heartbeat)
        .map_err(|e| e.to_string())?;
    *input_engine = Some(engine);
    
    let timer_display = session.timer().map(|t| t.remaining_display());
    overlay.show(&state, timer_display.as_deref(), signal_tx)
        .map_err(|e| e.to_string())?;
        
    tray.update_state(&state);
    
    Ok(())
}

fn perform_unlock(
    session: &mut SessionState,
    input_engine: &mut Option<InputEngine>,
    overlay: &mut OverlayManager,
    tray: &TrayManager,
) -> Result<(), String> {
    if let Some(mut engine) = input_engine.take() {
        engine.stop();
    }
    
    session.unlock();
    overlay.hide();
    tray.update_state(&LockState::Unlocked);
    
    Ok(())
}
