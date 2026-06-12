# CloaKey Roadmap

This document outlines the milestones and planned feature updates for CloaKey. We prioritize stability, trust, and usability. 

If you are interested in contributing, please review this roadmap to see where your efforts can align with the project's direction.

---

## 📍 V1: Foundation (Current Release)
The first phase focuses on establishing a rock-solid, secure, and reliable keyboard/mouse locking implementation on Windows.

-   [x] **Core Hooks:** Low-level Windows API keyboard and mouse hooks to block input without hanging the OS.
-   [x] **Interactive CLI:** A keyboard-navigable terminal UI (TUI) powered by `ratatui` (no mouse required).
-   [x] **Scriptable Interface:** CLI command structure (`cloak lock`, `cloak keyboard`, `cloak timer`) for automation.
-   [x] **Timed Protection:** Automatic unlocking after a user-specified duration (seconds, minutes, hours).
-   [x] **Emergency Recovery:** Unconditional physical override sequence: hold `CTRL + ALT + SHIFT` for 2 seconds.
-   [x] **System Startup:** Option to register CloaKey in the Windows Registry to run minimized on startup.
-   [x] **Status Interrogation:** Command `cloak status` to verify active modes and remaining lock durations.

---

## 🎨 V2: Interactive & Pet Protection (Q3 2026)
This phase introduces visual feedbacks, playful interaction safeguards, and floating indicators.

-   [ ] **Pet Mode (Bubble Pop UI):** A playful overlay screen where cat paws or toddler clicks trigger animated bubble-popping or simple visual effects instead of actual OS actions.
-   [ ] **Floating Key Visualizer:** An optional minimal overlay showing keys as they are typed (and blocked), providing visual confirmation of block performance.
-   [ ] **Input Absorption Animation:** Visual indicators showing inputs "bouncing off" the screen or boundaries in real-time.
-   [ ] **Enhanced Overlay Themes:** User-customizable translucent themes, styles, logo positioning, and opacity settings.

---

## 🍏 V3: Cross-Platform & Context Rules (Q1 2027)
Porting our low-level intercept hooks to macOS and Linux, and establishing application context awareness.

-   [ ] **macOS Support:** Native Apple CoreGraphics / EventTap hooks, packaging as a `.app` bundle, and plist launchd configurations.
-   [ ] **Linux Support:** Dual hooks backend for X11 (`XGrabKeyboard`/`XGrabPointer`) and Wayland environments (interception portal daemons).
-   [ ] **Application Profiles:** Define rules to lock input *only* when specific windows (e.g., active IDE, full-screen video, terminal runs) have focus.
-   [ ] **AI Agent Integration:** Integration daemons and APIs that allow AI coding environments (e.g., Claude Code, Devin) to request a temporary CloaKey lock programmatically while executing edits, and auto-unlock when complete.

---

## 🏢 V4: Enterprise & Policy Controls (Q3 2027)
Adding administration capabilities, multi-device support, and auditing for corporate environments.

-   [ ] **Enterprise Management Portal:** A centralized web console to monitor active lock statuses on remote corporate machines.
-   [ ] **Group Policies:** Push settings, shortcuts, and mandatory emergency unlock lengths from a centralized AD or MDM agent.
-   [ ] **Secure Audit Logging:** Log lock durations, active periods, and override activations for compliance reporting (without logging individual key values).
-   [ ] **Remote Administration:** Remotely trigger locks on active display terminals during presentations or off-hours.
