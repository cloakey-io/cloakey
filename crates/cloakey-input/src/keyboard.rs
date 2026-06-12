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

use crossbeam_channel::Sender;
use std::sync::Mutex;
use tracing::trace;
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_MENU, VK_RCONTROL, VK_RMENU,
    },
    UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, WM_KEYDOWN, WM_SYSKEYDOWN},
};

use cloakey_core::{LockMode, SafetySignal};

use crate::hooks::{block_event, pass_through};

/// Global lock mode — read by the hook callback to decide whether to block.
static LOCK_MODE: Mutex<LockMode> = Mutex::new(LockMode {
    keyboard_blocked: false,
    mouse_movement_blocked: false,
    mouse_clicks_blocked: false,
    mouse_scroll_blocked: false,
});

/// Global sender for safety signals.
static SIGNAL_TX: Mutex<Option<Sender<SafetySignal>>> = Mutex::new(None);

/// Initialize the keyboard hook globals.
///
/// Must be called before installing the keyboard hook.
pub(crate) fn init_keyboard_state(lock_mode: LockMode, signal_tx: Sender<SafetySignal>) {
    if let Ok(mut guard) = LOCK_MODE.lock() {
        *guard = lock_mode;
    }
    if let Ok(mut guard) = SIGNAL_TX.lock() {
        *guard = Some(signal_tx);
    }
}

/// Update the current lock mode (called when the lock state changes).
pub(crate) fn update_lock_mode(mode: LockMode) {
    if let Ok(mut guard) = LOCK_MODE.lock() {
        *guard = mode;
    }
}

/// Check if a Virtual Key code is a modifier key (Shift, Control, Alt, Win).
fn is_modifier_key(vk: u32) -> bool {
    matches!(
        vk,
        0x10 | 0x11 | 0x12 | 0x5B | 0x5C | 0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5
    )
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

    let kb_struct = l_param.0 as *const KBDLLHOOKSTRUCT;
    if !kb_struct.is_null() {
        let vk = (*kb_struct).vkCode;

        // Always pass through modifier keys so Windows keeps its physical key state up-to-date.
        // This is critical for GetAsyncKeyState to work for CTRL, ALT, and SHIFT.
        if is_modifier_key(vk) {
            return pass_through(n_code, w_param, l_param);
        }

        let msg = w_param.0 as u32;
        if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
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
                    if let Ok(guard) = SIGNAL_TX.lock() {
                        if let Some(ref tx) = *guard {
                            let _ = tx.send(SafetySignal::EmergencyUnlock);
                        }
                    }
                    return pass_through(n_code, w_param, l_param);
                }
            }
        }
    }

    let keyboard_blocked = LOCK_MODE
        .lock()
        .map(|m| m.keyboard_blocked)
        .unwrap_or(false);

    if !keyboard_blocked {
        return pass_through(n_code, w_param, l_param);
    }

    // Block keyboard event
    let msg = w_param.0 as u32;
    if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
        let kb_struct = l_param.0 as *const KBDLLHOOKSTRUCT;
        if !kb_struct.is_null() {
            let vk = (*kb_struct).vkCode;
            trace!("Blocking keyboard event: VK={:#x}", vk);
        }
    }

    block_event()
}
