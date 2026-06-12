//! Mouse hook callback and mouse event filtering.
//!
//! The `WH_MOUSE_LL` callback is called for EVERY mouse event system-wide.
//! It must return quickly and must not panic.
//!
//! Mouse event types and their handling per lock mode:
//!
//! | Event      | Unlocked | KeyboardLocked | MouseLocked | FullyLocked | GhostMode | TimedLock |
//! |------------|----------|----------------|-------------|-------------|-----------|-----------|
//! | WM_MOUSEMOVE | Pass   | Pass           | Block       | Block       | Pass      | Block     |
//! | WM_LBUTTONDOWN | Pass | Pass           | Block       | Block       | Block     | Block     |
//! | WM_RBUTTONDOWN | Pass | Pass           | Block       | Block       | Block     | Block     |
//! | WM_MBUTTONDOWN | Pass | Pass           | Block       | Block       | Block     | Block     |
//! | WM_MOUSEWHEEL  | Pass | Pass           | Block       | Block       | Block     | Block     |

use std::sync::{Mutex, OnceLock};

use tracing::trace;
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_MOUSEWHEEL,
        WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
    },
};

use cloakey_core::LockMode;

use crate::hooks::{block_event, pass_through};

/// Global lock mode for the mouse hook callback.
static MOUSE_LOCK_MODE: OnceLock<Mutex<LockMode>> = OnceLock::new();

/// Initialize the mouse hook globals.
pub(crate) fn init_mouse_state(lock_mode: LockMode) {
    let _ = MOUSE_LOCK_MODE.set(Mutex::new(lock_mode));
}

/// Update the current lock mode for the mouse hook.
pub(crate) fn update_mouse_lock_mode(mode: LockMode) {
    if let Some(m) = MOUSE_LOCK_MODE.get() {
        if let Ok(mut guard) = m.lock() {
            *guard = mode;
        }
    }
}

/// The `WH_MOUSE_LL` hook callback.
///
/// # Safety
/// This is a C-compatible extern callback. It must not panic or unwind.
pub(crate) unsafe extern "system" fn mouse_hook_callback(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    // nCode < 0 means we MUST pass to the next hook
    if n_code < 0 {
        return pass_through(n_code, w_param, l_param);
    }

    let mode = match MOUSE_LOCK_MODE.get().and_then(|m| m.lock().ok()) {
        Some(m) => m.clone(),
        None => return pass_through(n_code, w_param, l_param),
    };

    // If nothing is blocked for mouse, pass through immediately
    if !mode.mouse_movement_blocked && !mode.mouse_clicks_blocked && !mode.mouse_scroll_blocked {
        return pass_through(n_code, w_param, l_param);
    }

    let msg = w_param.0 as u32;

    match msg {
        WM_MOUSEMOVE => {
            if mode.mouse_movement_blocked {
                trace!("Blocking mouse movement");
                return block_event();
            }
        }

        WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN | WM_RBUTTONUP | WM_MBUTTONDOWN
        | WM_MBUTTONUP | WM_XBUTTONDOWN | WM_XBUTTONUP => {
            if mode.mouse_clicks_blocked {
                trace!("Blocking mouse click event: msg={:#x}", msg);
                return block_event();
            }
        }

        WM_MOUSEWHEEL => {
            if mode.mouse_scroll_blocked {
                trace!("Blocking mouse scroll event");
                return block_event();
            }
        }

        _ => {} // Unknown mouse message — pass through
    }

    pass_through(n_code, w_param, l_param)
}
