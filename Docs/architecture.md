# CloaKey Technical Architecture

Version: 1.0  
Product: CloaKey  
Status: Approved Architecture  
Primary Language: Rust  
Target Platform: Windows (Initial Release)  
Architecture Style: Modular Monolith  

---

## 🎯 Purpose & Goals

This document defines the technical architecture, module boundaries, system design principles, and implementation approach for CloaKey.

-   **Reliability:** No desktop freezes, lockouts, or system-wide thread blocking.
-   **Safety:** The system must never trap users. Emergency recovery always takes priority.
-   **Performance:** Hook Proc latencies under 1ms, startup time under 1 second.
-   **Low Resource Footprint:** Memory usage `< 50MB`, idle CPU usage `< 1%`.
-   **Future Scalability:** Platform abstraction layer to facilitate macOS and Linux ports.

---

## 🏛️ Architectural Principles

1.  **User Recovery Wins:** If there is ever a conflict between feature convenience and user recovery, **User Recovery always wins**. A user must always be able to regain control of their computer.
2.  **Input Suppression Only:** CloaKey intercepts and blocks inputs at the OS hook layer. It **never** records, saves, stores, or transmits keys, mouse events, or passwords.
3.  **Lightweight Core:** No heavy runtimes (e.g., Electron). Core components are written in native, high-performance systems Rust.
4.  **Platform Adaptability:** Keep the core coordination logic platform-independent. Only implementation adapters contain Windows/Unix-specific code.

---

## 🛠️ Technology Stack

-   **Core Language:** Rust (for memory safety, low-level hook efficiency, and thread safety).
-   **CLI Framework:** `clap` (for argument parsing and help output).
-   **Terminal UI:** `ratatui` & `crossterm` (for a keyboard-navigable BIOS-style console interface).
-   **Serialization:** `serde` & `toml` (for parsing User configs stored in `%APPDATA%\CloaKey\config.toml`).
-   **Diagnostics:** `tracing` (for local logging, with telemetry fully disabled).

---

## 📊 High-Level Architecture

```
                  ┌────────────────────────┐
                  │       CLI Layer        │
                  │   (ratatui / clap)     │
                  └───────────┬────────────┘
                              │
                              v
                  ┌────────────────────────┐
                  │    Application Core    │
                  └─────┬────────────┬─────┘
                        │            │
                        v            v
           ┌────────────────┐    ┌────────────────┐
           │  Input Engine  │    │Session Manager │
           └────────┬───────┘    └────────┬───────┘
                    │                     │
                    v                     v
           ┌──────────────────────────────────────┐
           │          Lock State Engine           │
           └──────────────────┬───────────────────┘
                              │
                              v
           ┌──────────────────────────────────────┐
           │            Overlay System            │
           └──────────────────┬───────────────────┘
                              │
                              v
           ┌──────────────────────────────────────┐
           │          Safety Controller           │
           └──────────────────────────────────────┘
```

---

## 📂 Core Modules & Responsibilities

### 1. CLI Module (`cloakey-cli`)
Parses incoming command arguments and handles active TUI menus. Translates options (e.g., `cloak lock`, `cloak timer 5m`) into states sent to the Session Manager.

### 2. Core Module (`cloakey-core`)
Orchestrates lock actions, transitions state machines, coordinates thread timers, and executes profile configurations. Has zero platform-specific dependencies.

### 3. Input Engine (`cloakey-input`)
Coordinates system-wide input hooks. On Windows, this translates to:
-   `SetWindowsHookExW` for `WH_KEYBOARD_LL` (Low-Level Keyboard Hook).
-   `SetWindowsHookExW` for `WH_MOUSE_LL` (Low-Level Mouse Hook).

When active, hooks return `1` (handled/blocked) to the OS message loop for intercepted events, preventing target windows from receiving them.

### 4. Lock State Engine
Tracks the active suppression state:
-   `Unlocked`
-   `KeyboardLocked`
-   `MouseLocked`
-   `FullyLocked`
-   `GhostMode` (Movement allowed; clicks/drags blocked)
-   `TimedLock`

---

## 🚨 Safety & Recovery Controller

The Safety Controller is the most critical component of CloaKey. It guarantees that the user can never be locked out of their system, even if the primary CLI process crashes or hangs.

### 1. Emergency Unlock Sequence
The Low-Level Keyboard Hook runs a continuous counter monitoring the physical states of:
`CTRL + ALT + SHIFT` (simultaneously held)

If held for **2 seconds**, the hook breaks, releases the raw pointers, calls `UnhookWindowsHookEx`, and returns control to the operating system shell immediately.

### 2. Deadman Switch
If the main coordination thread crashes, the input hook thread detects the socket or channel closure and automatically cleans up hooks to restore system access.
