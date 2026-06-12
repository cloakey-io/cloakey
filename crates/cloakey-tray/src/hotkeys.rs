use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};

pub struct HotkeyManager {
    _manager: GlobalHotKeyManager,
    hotkey_all: HotKey,
    hotkey_kbd: HotKey,
    hotkey_mouse: HotKey,
    hotkey_ghost: HotKey,
    hotkey_uncloak: HotKey,
}

impl HotkeyManager {
    /// Initialize the global hotkey manager and register CloaKey hotkeys.
    pub fn new() -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new().map_err(|e| e.to_string())?;
        
        // Setup hotkey definitions (Ctrl+Alt + L/K/M/G/U)
        let hotkey_all = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyL);
        let hotkey_kbd = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyK);
        let hotkey_mouse = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyM);
        let hotkey_ghost = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyG);
        let hotkey_uncloak = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyU);
        
        // Register hotkeys
        manager.register(hotkey_all).map_err(|e| format!("Failed to register Ctrl+Alt+L: {}", e))?;
        manager.register(hotkey_kbd).map_err(|e| format!("Failed to register Ctrl+Alt+K: {}", e))?;
        manager.register(hotkey_mouse).map_err(|e| format!("Failed to register Ctrl+Alt+M: {}", e))?;
        manager.register(hotkey_ghost).map_err(|e| format!("Failed to register Ctrl+Alt+G: {}", e))?;
        manager.register(hotkey_uncloak).map_err(|e| format!("Failed to register Ctrl+Alt+U: {}", e))?;
        
        Ok(Self {
            _manager: manager,
            hotkey_all,
            hotkey_kbd,
            hotkey_mouse,
            hotkey_ghost,
            hotkey_uncloak,
        })
    }
    
    /// Map a global hotkey event ID to a SafetySignal.
    pub fn handle_hotkey_event(&self, event_id: u32) -> Option<cloakey_core::SafetySignal> {
        if event_id == self.hotkey_all.id() {
            Some(cloakey_core::SafetySignal::HotkeyLockAll)
        } else if event_id == self.hotkey_kbd.id() {
            Some(cloakey_core::SafetySignal::HotkeyLockKeyboard)
        } else if event_id == self.hotkey_mouse.id() {
            Some(cloakey_core::SafetySignal::HotkeyLockMouse)
        } else if event_id == self.hotkey_ghost.id() {
            Some(cloakey_core::SafetySignal::HotkeyGhostMode)
        } else if event_id == self.hotkey_uncloak.id() {
            Some(cloakey_core::SafetySignal::HotkeyUnlock)
        } else {
            None
        }
    }
}
