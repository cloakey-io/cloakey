//! Keyboard hook callback and key filtering.
//!
//! The `WH_KEYBOARD_LL` callback is called for EVERY keyboard event system-wide.
//! It must return quickly and must not panic.
//!
//! Logic:
//! 1. Check if we should block (based on current `LockMode`).
//! 2. If locked: check for emergency unlock combo (CTRL+ALT+SHIFT held ≥ 2s).
//! 3. If emergency unlock detected: send signal, allow event through.
//! 4. Otherwise: block or pass based on lock mode.

use std::sync::{Mutex, OnceLock};
use crossbeam_channel::Sender;
use tracing::trace;
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_CONTROL, VK_LCONTROL, VK_RCONTROL, VK_MENU, VK_LMENU, VK_RMENU,
    },
    UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_SYSKEYDOWN},
};

use cloakey_core::{LockMode, SafetySignal};

use crate::hooks::{block_event, pass_through};

/// Global lock mode — read by the hook callback to decide whether to block.
static LOCK_MODE: OnceLock<Mutex<LockMode>> = OnceLock::new();

/// Global sender for safety signals.
static SIGNAL_TX: OnceLock<Sender<SafetySignal>> = OnceLock::new();

/// Initialize the keyboard hook globals.
///
/// Must be called before installing the keyboard hook.
pub(crate) fn init_keyboard_state(
    lock_mode: LockMode,
    signal_tx: Sender<SafetySignal>,
) {
    let _ = LOCK_MODE.set(Mutex::new(lock_mode));
    let _ = SIGNAL_TX.set(signal_tx);
}

/// Update the current lock mode (called when the lock state changes).
pub(crate) fn update_lock_mode(mode: LockMode) {
    if let Some(m) = LOCK_MODE.get() {
        if let Ok(mut guard) = m.lock() {
            *guard = mode;
        }
    }
}

/// The `WH_KEYBOARD_LL` hook callback.
///
/// # Safety
/// This is a C-compatible extern callback. It must not panic or unwind.
pub(crate) unsafe extern "system" fn keyboard_hook_callback(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    // nCode < 0 means we MUST pass to the next hook — do not process
    if n_code < 0 {
        return pass_through(n_code, w_param, l_param);
    }

    let msg = w_param.0 as u32;
    if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
        let kb_struct = l_param.0 as *const KBDLLHOOKSTRUCT;
        if !kb_struct.is_null() {
            let vk = (*kb_struct).vkCode;
            
            // Immediate "Uncloak" check: CTRL + ALT + U (0x55 is VK_U)
            if vk == 0x55 {
                let ctrl_held = (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0
                    || (GetAsyncKeyState(VK_LCONTROL.0 as i32) as u16 & 0x8000) != 0
                    || (GetAsyncKeyState(VK_RCONTROL.0 as i32) as u16 & 0x8000) != 0;
                let alt_held = (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0
                    || (GetAsyncKeyState(VK_LMENU.0 as i32) as u16 & 0x8000) != 0
                    || (GetAsyncKeyState(VK_RMENU.0 as i32) as u16 & 0x8000) != 0;
                
                if ctrl_held && alt_held {
                    trace!("Uncloak triggered via keyboard hook (Ctrl+Alt+U)");
                    if let Some(tx) = SIGNAL_TX.get() {
                        let _ = tx.send(SafetySignal::EmergencyUnlock);
                    }
                    return pass_through(n_code, w_param, l_param);
                }
            }
        }
    }

    let keyboard_blocked = LOCK_MODE
        .get()
        .and_then(|m| m.lock().ok())
        .map(|m| m.keyboard_blocked)
        .unwrap_or(false);

    if !keyboard_blocked {
        return pass_through(n_code, w_param, l_param);
    }

    // Block keyboard event
    if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
        let kb_struct = l_param.0 as *const KBDLLHOOKSTRUCT;
        if !kb_struct.is_null() {
            let vk = (*kb_struct).vkCode;
            trace!("Blocking keyboard event: VK={:#x}", vk);
        }
    }

    block_event()
}
