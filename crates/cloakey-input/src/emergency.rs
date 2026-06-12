//! Emergency unlock key tracking.
//!
//! Tracks the state of CTRL, ALT, and SHIFT keys to detect when all three
//! are held simultaneously for the required duration.
//!
//! This module re-exports the core `EmergencyUnlockTracker` and provides
//! Windows VK (Virtual Key) code constants for the three modifier keys.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_MENU, VK_RCONTROL, VK_RMENU,
    VK_RSHIFT, VK_SHIFT,
};

// Re-export from core
pub use cloakey_core::EmergencyUnlockTracker;

/// Check whether CTRL, ALT, and SHIFT are all currently held down.
///
/// Uses `GetAsyncKeyState` to check both left and right variants of each modifier.
///
/// # Safety
/// Safe to call from inside a hook callback. Returns instantly.
pub(crate) fn all_emergency_keys_held() -> bool {
    // SAFETY: GetAsyncKeyState is safe to call from any thread context.
    // High-order bit set means the key is currently pressed.
    unsafe {
        let ctrl_held = (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_LCONTROL.0 as i32) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_RCONTROL.0 as i32) as u16 & 0x8000) != 0;

        let alt_held = (GetAsyncKeyState(VK_MENU.0 as i32) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_LMENU.0 as i32) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_RMENU.0 as i32) as u16 & 0x8000) != 0;

        let shift_held = (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_LSHIFT.0 as i32) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_RSHIFT.0 as i32) as u16 & 0x8000) != 0;

        ctrl_held && alt_held && shift_held
    }
}
