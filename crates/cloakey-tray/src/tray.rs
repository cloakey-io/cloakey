use std::path::PathBuf;
use cloakey_core::LockState;
use tray_icon::{TrayIcon, TrayIconBuilder};
use muda::{Menu, MenuItem, PredefinedMenuItem};
use crate::icon::{load_icon_from_path, create_default_icon};

pub struct TrayManager {
    tray_icon: TrayIcon,
    item_lock_all: MenuItem,
    item_lock_kbd: MenuItem,
    item_lock_mouse: MenuItem,
    item_ghost_mode: MenuItem,
    item_unlock: MenuItem,
    item_quit: MenuItem,
    
    // Loaded icons
    icon_unlocked: tray_icon::Icon,
    icon_locked: tray_icon::Icon,
    icon_ghost: tray_icon::Icon,
}

impl TrayManager {
    /// Create a new TrayManager, initializing the tray icon and context menu.
    pub fn new() -> Result<Self, String> {
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_dir = exe_path.parent().ok_or_else(|| "Failed to get parent directory".to_string())?;
        
        let assets_dir = exe_dir.join("assets");
        
        // Load custom green key icon
        let icon_unlocked = load_icon_from_path(assets_dir.join("icon.png"))
            .unwrap_or_else(|_| create_default_icon([0, 180, 0, 255]));
            
        // Load custom red lock icon
        let icon_locked = load_icon_from_path(assets_dir.join("icon_locked.png"))
            .unwrap_or_else(|_| create_default_icon([220, 0, 0, 255]));
            
        // Purple circle for Ghost Mode fallback
        let icon_ghost = create_default_icon([120, 0, 180, 255]);

        let menu = Menu::new();
        let item_lock_all = MenuItem::new("Lock All (Ctrl+Alt+L)", true, None);
        let item_lock_kbd = MenuItem::new("Lock Keyboard (Ctrl+Alt+K)", true, None);
        let item_lock_mouse = MenuItem::new("Lock Mouse (Ctrl+Alt+M)", true, None);
        let item_ghost_mode = MenuItem::new("Ghost Mode (Ctrl+Alt+G)", true, None);
        let item_unlock = MenuItem::new("Uncloak (Ctrl+Alt+U)", false, None);
        let item_quit = MenuItem::new("Quit", true, None);

        menu.append_items(&[
            &item_lock_all,
            &item_lock_kbd,
            &item_lock_mouse,
            &item_ghost_mode,
            &PredefinedMenuItem::separator(),
            &item_unlock,
            &PredefinedMenuItem::separator(),
            &item_quit,
        ]).map_err(|e| e.to_string())?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("CloaKey — Unlocked")
            .with_icon(icon_unlocked.clone())
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            tray_icon,
            item_lock_all,
            item_lock_kbd,
            item_lock_mouse,
            item_ghost_mode,
            item_unlock,
            item_quit,
            icon_unlocked,
            icon_locked,
            icon_ghost,
        })
    }

    /// Update tray icon, tooltip, and menu item enabled states based on lock state.
    pub fn update_state(&self, state: &LockState) {
        let is_locked = *state != LockState::Unlocked;
        
        let _ = self.item_lock_all.set_enabled(!is_locked);
        let _ = self.item_lock_kbd.set_enabled(!is_locked);
        let _ = self.item_lock_mouse.set_enabled(!is_locked);
        let _ = self.item_ghost_mode.set_enabled(!is_locked);
        let _ = self.item_unlock.set_enabled(is_locked);
        
        let (icon, tooltip) = match state {
            LockState::Unlocked => (&self.icon_unlocked, "CloaKey — Unlocked"),
            LockState::KeyboardLocked => (&self.icon_locked, "CloaKey — Keyboard Locked"),
            LockState::MouseLocked => (&self.icon_locked, "CloaKey — Mouse Locked"),
            LockState::FullyLocked => (&self.icon_locked, "CloaKey — Fully Locked"),
            LockState::GhostMode => (&self.icon_ghost, "CloaKey — Ghost Mode"),
            LockState::TimedLock => (&self.icon_locked, "CloaKey — Timed Lock Active"),
        };
        
        let _ = self.tray_icon.set_icon(Some(icon.clone()));
        let _ = self.tray_icon.set_tooltip(Some(tooltip));
    }

    /// Translate a click event menu ID into a SafetySignal.
    pub fn handle_menu_event(&self, event_id: &muda::MenuId) -> Option<cloakey_core::SafetySignal> {
        if event_id == self.item_lock_all.id() {
            Some(cloakey_core::SafetySignal::HotkeyLockAll)
        } else if event_id == self.item_lock_kbd.id() {
            Some(cloakey_core::SafetySignal::HotkeyLockKeyboard)
        } else if event_id == self.item_lock_mouse.id() {
            Some(cloakey_core::SafetySignal::HotkeyLockMouse)
        } else if event_id == self.item_ghost_mode.id() {
            Some(cloakey_core::SafetySignal::HotkeyGhostMode)
        } else if event_id == self.item_unlock.id() {
            Some(cloakey_core::SafetySignal::HotkeyUnlock)
        } else if event_id == self.item_quit.id() {
            Some(cloakey_core::SafetySignal::Shutdown)
        } else {
            None
        }
    }
}
