# CloaKey Command Line & TUI Specification

This document defines the complete command-line interface (CLI), argument syntax, interactive terminal UI (TUI) menu layouts, and error behaviors for CloaKey.

---

## ⚡ CLI Design Principles

-   **Simple by Default:** Normal users only need to run a single command (`cloak`) to protect their screen.
-   **Interactive First:** If run without arguments, launch an interactive, keyboard-navigable terminal menu.
-   **Fully Scriptable:** Power users can bypass all TUI displays by passing direct arguments (e.g., `cloak lock`, `cloak timer 30s`).
-   **Context Awareness:** All active states must clearly show the active protection mode, status parameters, and emergency unlock instructions.

---

## 💻 Primary Invocation & Aliases

CloaKey registers two executable entry points during installation:
-   `cloak` (Primary command)
-   `cloakey` (Secondary alias)

Both executables resolve to the same underlying rust binary and support identical arguments.

---

## 🕹️ Interactive TUI Menu

Invoking `cloak` without arguments launches the ratatui-based console menu:

```text
=========================================
           CloaKey v1.0
    Workflow Protection System
=========================================

1. Lock Keyboard           [Mouse Active]
2. Lock Mouse              [Keyboard Active]
3. Lock Keyboard & Mouse   [Full Protection]
4. Ghost Mode              [Move cursor, No clicks]
5. Timed Lock              [Auto-unlock after time]
6. Settings                [Shortcuts, Overlays]
7. Help                    [List commands]
8. About                   [Version & license]

Q. Exit

Select option: 
```

### Navigation Keys
-   `1-8`: Select numeric options instantly.
-   `Arrow Up / Down` or `J / K`: Move selection highlighter.
-   `Enter`: Confirm selection.
-   `Esc` or `Q`: Exit TUI interface and restore terminal control.

---

## 🛠️ Direct Commands & Arguments

### 1. Lock Keyboard Only
Locks all keyboard keys and shortcuts, leaving mouse movement and clicks active.
```bash
cloak keyboard
# Alternative
cloak lock keyboard
```

### 2. Lock Mouse Only
Suppresses all mouse clicks, dragging, and scroll wheel events, leaving the keyboard fully active.
```bash
cloak mouse
```

### 3. Lock Both
Blocks both keyboard input and mouse interaction.
```bash
cloak lock
# Alternative
cloak both
```

### 4. Ghost Mode
Allows the cursor to be moved freely across screens, but suppresses all click, drag, scroll, and keyboard events. Useful for visual monitoring.
```bash
cloak ghost
```

### 5. Timed Lock
Automatically unlocks the system when the countdown timer reaches zero. Supports seconds (`s`), minutes (`m`), or hours (`h`). If no unit is provided, seconds are assumed.
```bash
# Lock for 60 seconds
cloak timer 60
# Lock for 15 minutes
cloak timer 15m
# Lock for 2 hours
cloak timer 2h
```

### 6. Session Status Check
Displays details of any active lock session.
```bash
cloak status
```
#### Output Example:
```text
================================
Current Status
================================
State:           Ghost Mode
Keyboard:        Blocked
Mouse Clicks:    Blocked
Mouse Movement:  Allowed
Remaining Time:  None
Unlock Shortcut: CTRL + ALT + SHIFT
```

### 7. Manual Unlock Command
Resets all hook interfaces and restores control.
```bash
cloak unlock
```

---

## ⚙️ Settings & Configuration Commands

### 1. Startup Integration
Configure CloaKey Windows Registry autostart keys.
```bash
cloak startup
```
Options configured in menu:
-   `Run On Startup` (Yes/No)
-   `Start Minimized` (Yes/No)
-   `Load Last Profile` (Yes/No)

### 2. Import/Export Settings
Backup or restore configurations.
```bash
# Export
cloak export
# Import
cloak import config.toml
# Reset to factory defaults
cloak reset
```
