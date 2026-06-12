//! Win32 overlay window — transparent, always-on-top, click-through status display.
//!
//! Window properties:
//! - `WS_EX_LAYERED`     — enables transparency and per-pixel alpha
//! - `WS_EX_TRANSPARENT` — mouse clicks pass through to windows below
//! - `WS_EX_TOPMOST`     — always rendered above other windows
//! - `WS_EX_TOOLWINDOW`  — hidden from the taskbar and Alt+Tab
//! - `WS_POPUP`          — no title bar, no borders

use std::sync::atomic::{AtomicIsize, Ordering};

use tracing::{debug, error, info};
use windows::Win32::{
    Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{
        BeginPaint, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect, InvalidateRect,
        SetBkMode, SetTextColor, DT_LEFT, DT_SINGLELINE, DT_VCENTER, PAINTSTRUCT, TRANSPARENT,
    },
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetSystemMetrics,
        PostMessageW, RegisterClassExW, SetLayeredWindowAttributes, ShowWindow, LWA_ALPHA,
        SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, WM_DESTROY, WM_PAINT, WM_USER, WNDCLASSEXW,
        WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
        WM_LBUTTONDBLCLK,
    },
};

use crate::error::OverlayError;

/// Custom window message to trigger a repaint with new text (reserved for future use).
#[allow(dead_code)]
const WM_UPDATE_TEXT: u32 = WM_USER + 1;
/// Custom window message to destroy the window from another thread.
const WM_CLOSE_OVERLAY: u32 = WM_USER + 2;
/// Custom window message for system tray callback notifications.
const WM_TRAY_CALLBACK: u32 = WM_USER + 10;

/// Global safety signal sender to trigger unlock when double-clicking the tray icon.
static SAFETY_SENDER: std::sync::Mutex<Option<crossbeam_channel::Sender<cloakey_core::SafetySignal>>> =
    std::sync::Mutex::new(None);

/// Global handle to the overlay window (0 = no window).
pub(crate) static OVERLAY_HWND: AtomicIsize = AtomicIsize::new(0);

/// The status text to display (stored as a global for access from WM_PAINT).
static DISPLAY_TEXT: std::sync::OnceLock<std::sync::Mutex<String>> = std::sync::OnceLock::new();

/// Create and show the overlay window.
///
/// `centered = true` → Standard mode (center screen).
/// `centered = false` → Minimal mode (bottom-right corner).
///
/// # Safety
/// Must be called from a thread with a message pump.
pub(crate) unsafe fn create_overlay_window(
    centered: bool,
    signal_tx: crossbeam_channel::Sender<cloakey_core::SafetySignal>,
) -> Result<HWND, OverlayError> {
    let _ = DISPLAY_TEXT.set(std::sync::Mutex::new(String::new()));

    // Register window class
    let class_name: Vec<u16> = "CloaKeyOverlay\0".encode_utf16().collect();
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(overlay_wnd_proc),
        hInstance: HINSTANCE::default(),
        lpszClassName: windows::core::PCWSTR(class_name.as_ptr()),
        ..Default::default()
    };

    let atom = RegisterClassExW(&wc);
    if atom == 0 {
        let err = windows::Win32::Foundation::GetLastError();
        // Ignore ERROR_CLASS_ALREADY_EXISTS (1410) — safe to reuse
        if err.0 != 1410 {
            return Err(OverlayError::ClassRegisterFailed { win32_error: err.0 });
        }
    }

    // Determine window size and position
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);

    let win_w: i32 = if centered { 480 } else { 320 };
    let win_h: i32 = if centered { 60 } else { 36 };

    let (x, y) = if centered {
        ((screen_w - win_w) / 2, (screen_h - win_h) / 2)
    } else {
        // Bottom-right corner with margin
        (screen_w - win_w - 20, screen_h - win_h - 60)
    };

    let ex_style = WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW;

    let window_title: Vec<u16> = "CloaKey\0".encode_utf16().collect();

    let hwnd = CreateWindowExW(
        ex_style,
        windows::core::PCWSTR(class_name.as_ptr()),
        windows::core::PCWSTR(window_title.as_ptr()),
        WS_POPUP,
        x,
        y,
        win_w,
        win_h,
        None,
        None,
        HINSTANCE::default(),
        None,
    );

    match hwnd {
        Ok(h) if !h.is_invalid() => {
            // Store the safety sender for double-click tray unlock
            if let Ok(mut guard) = SAFETY_SENDER.lock() {
                *guard = Some(signal_tx);
            }
            // 85% opacity (216/255)
            let _ = SetLayeredWindowAttributes(h, COLORREF(0), 216, LWA_ALPHA);
            let _ = ShowWindow(h, SW_SHOW);
            OVERLAY_HWND.store(h.0 as isize, Ordering::Release);
            register_tray_icon(h);
            info!("Overlay window created (centered={})", centered);
            Ok(h)
        }
        _ => {
            let err = windows::Win32::Foundation::GetLastError();
            error!("Failed to create overlay window: Win32 error {}", err.0);
            Err(OverlayError::WindowCreateFailed { win32_error: err.0 })
        }
    }
}

/// Update the text displayed in the overlay and trigger a repaint.
pub(crate) fn update_overlay_text(text: &str) {
    if let Some(m) = DISPLAY_TEXT.get() {
        if let Ok(mut s) = m.lock() {
            *s = text.to_string();
        }
    }
    let hwnd_raw = OVERLAY_HWND.load(Ordering::Acquire);
    if hwnd_raw != 0 {
        unsafe {
            let hwnd = HWND(hwnd_raw as _);
            let _ = InvalidateRect(hwnd, None, true);
        }
    }
}

/// Destroy the overlay window.
pub(crate) fn destroy_overlay() {
    let hwnd_raw = OVERLAY_HWND.swap(0, Ordering::AcqRel);
    if hwnd_raw != 0 {
        unsafe {
            let hwnd = HWND(hwnd_raw as _);
            let _ = PostMessageW(hwnd, WM_CLOSE_OVERLAY, WPARAM(0), LPARAM(0));
        }
        debug!("Overlay destroy message sent");
    }
}

/// Window procedure for the overlay window.
unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            // Transparent background mode for text
            SetBkMode(hdc, TRANSPARENT);
            // White text
            SetTextColor(hdc, COLORREF(0x00FFFFFF));

            // Get client rect
            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);

            // Dark semi-transparent background
            let bg_brush = CreateSolidBrush(COLORREF(0x00222222));
            FillRect(hdc, &rect, bg_brush);
            let _ = DeleteObject(bg_brush);

            // Retrieve the status text
            let text = DISPLAY_TEXT
                .get()
                .and_then(|m| m.lock().ok())
                .map(|s| s.clone())
                .unwrap_or_else(|| "\u{1F512} CloaKey Active".to_string());

            // Add padding inset
            let mut text_rect = RECT {
                left: rect.left + 10,
                top: rect.top + 6,
                right: rect.right - 10,
                bottom: rect.bottom - 6,
            };

            // DrawTextW requires &mut [u16]
            let mut text_wide: Vec<u16> = text.encode_utf16().collect();
            DrawTextW(
                hdc,
                &mut text_wide,
                &mut text_rect,
                DT_LEFT | DT_VCENTER | DT_SINGLELINE,
            );

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        WM_TRAY_CALLBACK => {
            let event = l_param.0 as u32;
            if event == WM_LBUTTONDBLCLK {
                if let Ok(guard) = SAFETY_SENDER.lock() {
                    if let Some(ref tx) = *guard {
                        let _ = tx.send(cloakey_core::SafetySignal::EmergencyUnlock);
                    }
                }
            }
            LRESULT(0)
        }

        WM_CLOSE_OVERLAY => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }

        WM_DESTROY => {
            unregister_tray_icon(hwnd);
            // Clear the safety sender
            if let Ok(mut guard) = SAFETY_SENDER.lock() {
                *guard = None;
            }
            OVERLAY_HWND.store(0, Ordering::Release);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, w_param, l_param),
    }
}

unsafe fn register_tray_icon(hwnd: HWND) {
    use windows::Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NOTIFYICONDATAW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{LoadIconW, IDI_SHIELD};

    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAY_CALLBACK,
        ..Default::default()
    };

    if let Ok(h_icon) = LoadIconW(None, IDI_SHIELD) {
        nid.hIcon = h_icon;
    }

    let tip = "CloaKey — Protection Active\0";
    let tip_wide: Vec<u16> = tip.encode_utf16().collect();
    let len = tip_wide.len().min(127);
    nid.szTip[..len].copy_from_slice(&tip_wide[..len]);

    let _ = Shell_NotifyIconW(NIM_ADD, &nid);
}

unsafe fn unregister_tray_icon(hwnd: HWND) {
    use windows::Win32::UI::Shell::{Shell_NotifyIconW, NIM_DELETE, NOTIFYICONDATAW};

    let nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        ..Default::default()
    };

    let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
}
