//! Win32 overlay window — transparent, always-on-top, click-through status display.
//!
//! Window properties:
//! - `WS_EX_LAYERED`     — enables transparency and per-pixel alpha
//! - `WS_EX_TRANSPARENT` — mouse clicks pass through to windows below
//! - `WS_EX_TOPMOST`     — always rendered above other windows
//! - `WS_EX_TOOLWINDOW`  — hidden from the taskbar and Alt+Tab
//! - `WS_POPUP`          — no title bar, no borders

use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Mutex, OnceLock};

use image::RgbaImage;
use tracing::{debug, error, info};
use windows::Win32::{
    Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{
        BeginPaint, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect, InvalidateRect,
        SelectObject, SetBkMode, SetTextColor, DT_CENTER, DT_SINGLELINE, DT_VCENTER, PAINTSTRUCT,
        TRANSPARENT,
    },
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, GetSystemMetrics,
        PostMessageW, RegisterClassExW, SetLayeredWindowAttributes, ShowWindow, LWA_ALPHA,
        LWA_COLORKEY, SM_CXSCREEN, SM_CYSCREEN, SW_SHOW, WM_DESTROY, WM_LBUTTONDBLCLK, WM_PAINT,
        WM_USER, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
        WS_POPUP,
    },
};

use crate::error::OverlayError;
use cloakey_core::{LockState, SafetySignal};
use cloakey_settings::OverlayMode;

/// Custom window message to destroy the window from another thread.
const WM_CLOSE_OVERLAY: u32 = WM_USER + 2;
/// Custom window message for system tray callback notifications.
const WM_TRAY_CALLBACK: u32 = WM_USER + 10;

/// Global safety signal sender to trigger unlock when double-clicking the tray icon.
static SAFETY_SENDER: Mutex<Option<crossbeam_channel::Sender<SafetySignal>>> = Mutex::new(None);

/// Global handle to the overlay window (0 = no window).
pub(crate) static OVERLAY_HWND: AtomicIsize = AtomicIsize::new(0);

/// The status text to display (stored as a global for access from WM_PAINT).
static DISPLAY_TEXT: OnceLock<Mutex<String>> = OnceLock::new();

/// The active lock state and overlay mode (stored as globals for rendering choices).
static CURRENT_STATE: Mutex<LockState> = Mutex::new(LockState::Unlocked);
static OVERLAY_MODE: Mutex<OverlayMode> = Mutex::new(OverlayMode::Standard);

/// Embedded logo PNG decoded once at runtime.
static EMBEDDED_LOGO: OnceLock<RgbaImage> = OnceLock::new();

/// Set the current lock state.
pub(crate) fn set_overlay_state(state: LockState) {
    if let Ok(mut guard) = CURRENT_STATE.lock() {
        *guard = state;
    }
}

/// Set the current overlay mode.
pub(crate) fn set_overlay_mode(mode: OverlayMode) {
    if let Ok(mut guard) = OVERLAY_MODE.lock() {
        *guard = mode;
    }
}

/// Retrieve the decoded logo.
fn get_logo_image() -> &'static RgbaImage {
    EMBEDDED_LOGO.get_or_init(|| {
        let png_bytes = include_bytes!("../../../CloaKey logo no bg.png");
        image::load_from_memory(png_bytes)
            .expect("Failed to load embedded CloaKey logo")
            .to_rgba8()
    })
}

/// Helper to render the decoded logo onto an HDC using GdiAlphaBlend.
unsafe fn draw_logo_to_dc(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    dest_x: i32,
    dest_y: i32,
    dest_w: i32,
    dest_h: i32,
) {
    use std::ptr;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GdiAlphaBlend, SelectObject,
        AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION,
        DIB_RGB_COLORS,
    };

    let logo = get_logo_image();
    let src_w = logo.width() as i32;
    let src_h = logo.height() as i32;

    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: src_w,
            biHeight: -src_h, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut bits: *mut std::ffi::c_void = ptr::null_mut();
    let hbitmap = match CreateDIBSection(None, &bmi, DIB_RGB_COLORS, &mut bits, None, 0) {
        Ok(h) => h,
        Err(_) => return,
    };

    if !bits.is_null() && !hbitmap.is_invalid() {
        let bits_slice =
            std::slice::from_raw_parts_mut(bits as *mut u8, (src_w * src_h * 4) as usize);
        let raw_pixels = logo.as_raw();
        // Convert RGBA to BGRA
        for i in 0..(src_w * src_h) as usize {
            let offset = i * 4;
            bits_slice[offset] = raw_pixels[offset + 2]; // B
            bits_slice[offset + 1] = raw_pixels[offset + 1]; // G
            bits_slice[offset + 2] = raw_pixels[offset]; // R
            bits_slice[offset + 3] = raw_pixels[offset + 3]; // A
        }

        let mem_dc = CreateCompatibleDC(hdc);
        let old_bitmap = SelectObject(mem_dc, hbitmap);

        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        let _ = GdiAlphaBlend(
            hdc, dest_x, dest_y, dest_w, dest_h, mem_dc, 0, 0, src_w, src_h, blend,
        );

        SelectObject(mem_dc, old_bitmap);
        let _ = DeleteDC(mem_dc);
    }

    if !hbitmap.is_invalid() {
        let _ = DeleteObject(hbitmap);
    }
}

/// Create and show the overlay window.
///
/// `centered = true` → Standard/TimedLock mode (center screen).
/// `centered = false` → Minimal mode (bottom-right corner).
///
/// # Safety
/// Must be called from a thread with a message pump.
pub(crate) unsafe fn create_overlay_window(
    _centered: bool,
    signal_tx: crossbeam_channel::Sender<SafetySignal>,
) -> Result<HWND, OverlayError> {
    let _ = DISPLAY_TEXT.set(Mutex::new(String::new()));

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
        // Ignore ERROR_CLASS_ALREADY_EXISTS (1410)
        if err.0 != 1410 {
            return Err(OverlayError::ClassRegisterFailed { win32_error: err.0 });
        }
    }

    // Determine window size and position based on current state and mode
    let screen_w = GetSystemMetrics(SM_CXSCREEN);
    let screen_h = GetSystemMetrics(SM_CYSCREEN);

    let state = if let Ok(guard) = CURRENT_STATE.lock() {
        guard.clone()
    } else {
        LockState::Unlocked
    };
    let mode = if let Ok(guard) = OVERLAY_MODE.lock() {
        guard.clone()
    } else {
        OverlayMode::Standard
    };

    let (win_w, win_h) = if mode == OverlayMode::Minimal {
        (200, 50)
    } else if state == LockState::TimedLock {
        (256, 320)
    } else {
        (256, 256)
    };

    let (x, y) = if mode == OverlayMode::Minimal {
        (screen_w - win_w - 20, screen_h - win_h - 60)
    } else {
        ((screen_w - win_w) / 2, ((screen_h - win_h) / 2) - 30)
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

            // Set Layered Window Attributes depending on Minimal vs Standard/TimedLock
            if mode == OverlayMode::Minimal {
                // 10% opacity black-tinted background (alpha = 25)
                let _ = SetLayeredWindowAttributes(h, COLORREF(0), 25, LWA_ALPHA);
            } else {
                // Color key transparency: magenta (0x00FF00FF) is transparent
                let _ = SetLayeredWindowAttributes(h, COLORREF(0x00FF00FF), 255, LWA_COLORKEY);
            }

            let _ = ShowWindow(h, SW_SHOW);
            OVERLAY_HWND.store(h.0 as isize, Ordering::Release);
            register_tray_icon(h);
            info!(
                "Overlay window created (mode={:?}, state={:?})",
                mode, state
            );
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

            let state = if let Ok(guard) = CURRENT_STATE.lock() {
                guard.clone()
            } else {
                LockState::Unlocked
            };
            let mode = if let Ok(guard) = OVERLAY_MODE.lock() {
                guard.clone()
            } else {
                OverlayMode::Standard
            };

            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);

            if mode == OverlayMode::Minimal {
                // Minimal Mode: 10% blurred background, only "CloaKey" text in cyberpunk font.
                let bg_brush = CreateSolidBrush(COLORREF(0x00111111));
                FillRect(hdc, &rect, bg_brush);
                let _ = DeleteObject(bg_brush);

                SetBkMode(hdc, TRANSPARENT);
                // Neon Cyan text: 0x00FFFF00 (BGR)
                SetTextColor(hdc, COLORREF(0x00FFFF00));

                let font_name: Vec<u16> = "Consolas\0".encode_utf16().collect();
                use windows::Win32::Graphics::Gdi::CreateFontW;
                let font = CreateFontW(
                    28, // Height
                    0,
                    0,
                    0,
                    700, // FW_BOLD
                    0,
                    0,
                    0,
                    0, // ANSI_CHARSET
                    0, // OUT_DEFAULT_PRECIS
                    0, // CLIP_DEFAULT_PRECIS
                    5, // CLEARTYPE_QUALITY
                    0,
                    windows::core::PCWSTR(font_name.as_ptr()),
                );
                let old_font = SelectObject(hdc, font);

                let text = "CloaKey";
                let mut text_wide: Vec<u16> = text.encode_utf16().collect();
                DrawTextW(
                    hdc,
                    &mut text_wide,
                    &mut rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );

                SelectObject(hdc, old_font);
                let _ = DeleteObject(font);
            } else if state == LockState::TimedLock {
                // Timed Lock Mode: Transparent background, logo centered, countdown underneath.
                let bg_brush = CreateSolidBrush(COLORREF(0x00FF00FF));
                FillRect(hdc, &rect, bg_brush);
                let _ = DeleteObject(bg_brush);

                // Draw logo in the upper portion
                draw_logo_to_dc(hdc, 0, 0, 256, 256);

                SetBkMode(hdc, TRANSPARENT);
                // Neon Cyan text: 0x00FFFF00 (BGR)
                SetTextColor(hdc, COLORREF(0x00FFFF00));

                let raw_text = DISPLAY_TEXT
                    .get()
                    .and_then(|m| m.lock().ok())
                    .map(|s| s.clone())
                    .unwrap_or_default();

                // Extract just the countdown timer string (e.g. "4m 32s remaining")
                let display_time = if let Some(dash_idx) = raw_text.find("—") {
                    raw_text[dash_idx + 1..].trim().to_string()
                } else {
                    raw_text.clone()
                };

                let font_name: Vec<u16> = "Consolas\0".encode_utf16().collect();
                use windows::Win32::Graphics::Gdi::CreateFontW;
                let font = CreateFontW(
                    26, // Height
                    0,
                    0,
                    0,
                    700, // FW_BOLD
                    0,
                    0,
                    0,
                    0, // ANSI_CHARSET
                    0, // OUT_DEFAULT_PRECIS
                    0, // CLIP_DEFAULT_PRECIS
                    5, // CLEARTYPE_QUALITY
                    0,
                    windows::core::PCWSTR(font_name.as_ptr()),
                );
                let old_font = SelectObject(hdc, font);

                let mut text_rect = RECT {
                    left: 0,
                    top: 260,
                    right: 256,
                    bottom: 320,
                };
                let mut text_wide: Vec<u16> = display_time.encode_utf16().collect();
                DrawTextW(
                    hdc,
                    &mut text_wide,
                    &mut text_rect,
                    DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                );

                SelectObject(hdc, old_font);
                let _ = DeleteObject(font);
            } else {
                // Standard Mode: Centered logo only, transparent background
                let bg_brush = CreateSolidBrush(COLORREF(0x00FF00FF));
                FillRect(hdc, &rect, bg_brush);
                let _ = DeleteObject(bg_brush);

                draw_logo_to_dc(hdc, 0, 0, 256, 256);
            }

            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        WM_TRAY_CALLBACK => {
            let event = l_param.0 as u32;
            if event == WM_LBUTTONDBLCLK {
                if let Ok(guard) = SAFETY_SENDER.lock() {
                    if let Some(ref tx) = *guard {
                        let _ = tx.send(SafetySignal::EmergencyUnlock);
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
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Shell::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NOTIFYICONDATAW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{LoadImageW, IMAGE_ICON, LR_DEFAULTSIZE};

    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
        uCallbackMessage: WM_TRAY_CALLBACK,
        ..Default::default()
    };

    let h_instance = GetModuleHandleW(None).unwrap_or_default();
    // Load the first icon resource (index 1) embedded inside the executable
    if let Ok(h_icon) = LoadImageW(
        h_instance,
        windows::core::PCWSTR(1 as _),
        IMAGE_ICON,
        0,
        0,
        LR_DEFAULTSIZE,
    ) {
        nid.hIcon = windows::Win32::UI::WindowsAndMessaging::HICON(h_icon.0);
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
