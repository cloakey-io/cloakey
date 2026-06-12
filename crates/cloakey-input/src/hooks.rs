//! Win32 low-level hook management.
//!
//! Provides safe wrappers around `SetWindowsHookExW` and `UnhookWindowsHookEx`
//! for installing global keyboard and mouse hooks.
//!
//! # Safety
//! The hook callbacks (`WH_KEYBOARD_LL` and `WH_MOUSE_LL`) run in the thread
//! that processes messages for the hook. They MUST return quickly (< 50ms) and
//! MUST NOT panic. All critical logic is kept minimal.

use std::sync::atomic::{AtomicIsize, Ordering};

use tracing::{debug, error, info};
use windows::Win32::{
    Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, PeekMessageW, PostQuitMessage, SetWindowsHookExW,
        TranslateMessage, DispatchMessageW, UnhookWindowsHookEx,
        HHOOK, MSG, PM_REMOVE, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_QUIT,
    },
};

use crate::error::InputError;

// Global hook handles stored atomically so the hook callbacks can read them.
// We use AtomicIsize to store the HHOOK raw value (isize on Windows).
pub(crate) static KEYBOARD_HOOK: AtomicIsize = AtomicIsize::new(0);
pub(crate) static MOUSE_HOOK: AtomicIsize = AtomicIsize::new(0);

/// Install the global keyboard low-level hook.
///
/// The hook callback is defined in `keyboard.rs`.
///
/// # Safety
/// Must be called from a thread with a message pump (see `run_message_pump`).
pub(crate) unsafe fn install_keyboard_hook(
    callback: windows::Win32::UI::WindowsAndMessaging::HOOKPROC,
) -> Result<HHOOK, InputError> {
    let hook = SetWindowsHookExW(WH_KEYBOARD_LL, callback, HINSTANCE::default(), 0);
    match hook {
        Ok(h) if !h.is_invalid() => {
            KEYBOARD_HOOK.store(h.0 as isize, Ordering::Release);
            info!("Keyboard hook installed successfully");
            Ok(h)
        }
        _ => {
            let err = windows::Win32::Foundation::GetLastError();
            error!("Failed to install keyboard hook: Win32 error {}", err.0);
            Err(InputError::HookInstallFailed {
                hook_type: "WH_KEYBOARD_LL",
                win32_error: err.0,
            })
        }
    }
}

/// Install the global mouse low-level hook.
///
/// The hook callback is defined in `mouse.rs`.
///
/// # Safety
/// Must be called from a thread with a message pump (see `run_message_pump`).
pub(crate) unsafe fn install_mouse_hook(
    callback: windows::Win32::UI::WindowsAndMessaging::HOOKPROC,
) -> Result<HHOOK, InputError> {
    let hook = SetWindowsHookExW(WH_MOUSE_LL, callback, HINSTANCE::default(), 0);
    match hook {
        Ok(h) if !h.is_invalid() => {
            MOUSE_HOOK.store(h.0 as isize, Ordering::Release);
            info!("Mouse hook installed successfully");
            Ok(h)
        }
        _ => {
            let err = windows::Win32::Foundation::GetLastError();
            error!("Failed to install mouse hook: Win32 error {}", err.0);
            Err(InputError::HookInstallFailed {
                hook_type: "WH_MOUSE_LL",
                win32_error: err.0,
            })
        }
    }
}

/// Remove a hook handle, logging any failure.
///
/// This is designed to be callable from `Drop` — it must not panic.
pub(crate) unsafe fn remove_hook(hook: HHOOK, hook_type: &'static str) {
    if hook.is_invalid() {
        return;
    }
    if UnhookWindowsHookEx(hook).is_err() {
        let err = windows::Win32::Foundation::GetLastError();
        error!(
            "Failed to remove {} hook (Win32 error: {}). Input may remain active.",
            hook_type, err.0
        );
    } else {
        debug!("{} hook removed", hook_type);
    }
}

/// Pass the event to the next hook in the chain (allow it through).
///
/// # Safety
/// Must be called inside a hook callback with valid nCode, wParam, lParam.
#[inline]
pub(crate) unsafe fn pass_through(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    CallNextHookEx(HHOOK::default(), n_code, w_param, l_param)
}

/// Block the event (swallow it — do not pass to the next hook or to applications).
///
/// Returning `LRESULT(1)` from a low-level hook callback blocks the event.
#[inline]
pub(crate) fn block_event() -> LRESULT {
    LRESULT(1)
}

/// Process all pending Win32 messages (non-blocking).
///
/// Uses `PeekMessage` to drain the message queue without blocking. This allows
/// the calling loop to pulse the heartbeat, check the stop flag, and call this
/// function again on the next iteration.
///
/// Returns `true` if WM_QUIT was received (caller should exit).
///
/// # Safety
/// Must be called from the thread where hooks are installed.
pub(crate) unsafe fn run_message_pump() -> bool {
    let mut msg = MSG::default();
    while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
        if msg.message == WM_QUIT {
            debug!("WM_QUIT received in message pump");
            return true;
        }
        let _ = TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
    false
}

/// Signal the message pump to exit (called during shutdown).
///
/// # Safety
/// Safe to call from any thread.
pub(crate) unsafe fn stop_message_pump() {
    PostQuitMessage(0);
}
