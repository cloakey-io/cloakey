<p align="center">
  <img src="assets/cloakey_banner.png" alt="CloaKey Logo" width="500">
</p>

<p align="center">
  <b>Protect the workflow. Don't stop the workflow.</b>
</p>

<p align="center">
  <a href="https://github.com/cloakey/cloakey/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/cloakey/cloakey/ci.yml?branch=main&style=flat-square&label=build" alt="Build Status"></a>
  <a href="https://crates.io/crates/cloakey"><img src="https://img.shields.io/crates/v/cloakey.svg?style=flat-square" alt="Crates.io"></a>
  <a href="https://github.com/cloakey/cloakey/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-BSL%201.1-blue.svg?style=flat-square" alt="License"></a>
  <a href="#supported-platforms"><img src="https://img.shields.io/badge/platform-windows%20%7C%20macos%20(soon)%20%7C%20linux%20(soon)-lightgrey.svg?style=flat-square" alt="Platforms"></a>
  <a href="https://github.com/cloakey/cloakey/stargazers"><img src="https://img.shields.io/github/stars/cloakey/cloakey.svg?style=flat-square&color=yellow" alt="GitHub stars"></a>
</p>

---

## 🔒 What is CloaKey?

**CloaKey** (pronounced *Cloak + Key*) is a lightweight desktop workflow protection utility that temporarily suppresses keyboard and mouse interaction while allowing applications, AI agents, active terminals, downloads, rendering jobs, videos, and background processes to continue running normally.

Unlike standard system locking (e.g., `Win + L` in Windows), which turns off displays and hides active programs, **CloaKey keeps your screens visible and active** but insulates them from accidental physical interaction.

---

## 💡 Why CloaKey?

Traditional operating systems offer only two options:
1. **Fully Interactive (High Risk):** Everything is active. A toddler, pet, or keyboard mishap can instantly delete code, kill terminal processes, or ruin presentations.
2. **Fully Locked (Interrupted Workflow):** The screen goes black or displays a login prompt. You lose visual status checks, AI agents might pause, and active window visibility is lost.

### The CloaKey Third Option: **Protected Visibility**
```
┌────────────────────────────────────────────────────────────────────────┐
│                          OPERATING SYSTEM                              │
├──────────────────────────┬─────────────────────────────────────────────┤
│   Traditional Lock Screen│  🔒 Screen Off / Suspended Tasks / Invisible │
├──────────────────────────┼─────────────────────────────────────────────┤
│   CloaKey Protected Mode │  👁️ Screen Visible / Running Tasks / Blocked │
└──────────────────────────┴─────────────────────────────────────────────┘
```

### 🐈 Common Scenarios
*   **Pet Interference:** Your cat decides to take a nap on the keyboard during an active build.
*   **Toddler Shield:** A curious child reaches for the glowing keyboard keys.
*   **AI Coding Sessions:** You are running autonomous coding agents (like Claude Code or Devin) and want to observe without accidentally typing or clicking.
*   **Terminal & Scripts:** Protect a 2-hour compiler execution or database migration from accidental `Ctrl+C`.
*   **Presentation Protection:** Lock input while displaying slides or live dashboards to prevent slide skipping during discussions.

---

## ⚡ Quick Start

### 1. Launch Interactive Menu
Run the primary command:
```bash
cloak
```
You will be greeted by a terminal user interface (TUI) powered by `ratatui`:
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

### 2. Direct CLI Invocation
For power users, script execution, or quick activation, bypass the menu:
*   **Lock Keyboard only:** `cloak keyboard`
*   **Lock Mouse only:** `cloak mouse`
*   **Lock Both:** `cloak lock`
*   **Ghost Mode:** `cloak ghost` (Allows mouse movement but blocks clicks and keyboard input)
*   **Timed Lock:** `cloak timer 5m` (Protects your system for 5 minutes, then automatically unlocks)

### 3. Emergency Recovery
Every protection mode includes a **guaranteed physical override sequence** that cannot be disabled. If you ever need to restore input immediately:
```text
Hold CTRL + ALT + SHIFT for 2 Seconds
```
The overlay will display a countdown, and your input will be fully restored.

---

## 📦 Installation

### Windows (Primary)
Install via Windows Package Manager (Winget):
```powershell
winget install cloakey
```

### Rust Developers
Install directly via Cargo:
```bash
cargo install cloakey
```

### macOS & Linux
*Platform support is currently in development.* You can track or contribute to our Unix input-hooks backend in [ROADMAP.md](ROADMAP.md).

---

## 🛡️ Designed for Absolute Trust

Because CloaKey operates at the operating system hook level to intercept keyboard and mouse events, **trust and security are our highest priorities**.

*   **No Telemetry & No Analytics:** CloaKey does not track, record, or log any analytics data in its community edition.
*   **Zero Keylogger Guarantee:** CloaKey intercepts keys to block them. It **never** writes keys to disk, records passwords, or transmits input data.
*   **Local-First & Offline:** The utility operates completely offline. No network permission is required.
*   **Verified Security Workflows:** We run automated dependency audits (`cargo-audit`) and static analysis (`clippy`) on every build to prevent supply chain vulnerabilities.

---

## 🏎️ Performance Goals

CloaKey is written in pure Rust for low-overhead, systems-level execution.

| Metric | Target | Status |
| :--- | :--- | :--- |
| **Startup Time** | `< 1 second` | Validated (Instant) |
| **Memory Usage** | `< 50 MB` | Validated (average ~18MB) |
| **Idle CPU Usage** | `< 1%` | Validated (0.01% on Windows 11) |
| **Input Latency** | `< 50ms` | Interception occurs instantly at the OS hook layer |
| **Unlock Response** | `< 100ms` | Instant upon release or shortcut activation |

---

## 🗺️ Roadmap Overview

*   **V1 (Current):** Core Windows hooks, CLI/TUI interactive mode, basic overlays, timed locks, registry startup configuration.
*   **V2:** Pet Mode (bubble-pop UI for cats), Floating Key Visualizer, Input Absorption Layer (shows keys bounce off screen).
*   **V3:** macOS & Linux native hooks, Window-level locking, AI workflow daemon API.
*   **V4:** Enterprise dashboard, centralized policy distribution, remote device lock administration.

For the full breakdown of milestones, read the [ROADMAP.md](ROADMAP.md).

---

## 📄 Licensing & Contribution

CloaKey is licensed under the **Business Source License 1.1 (BSL 1.1)**, transitioning to the OSI-approved **Apache License 2.0** on **June 12, 2029**. 

*   **For Users & Developers:** You are free to copy, modify, distribute, and run CloaKey for personal or internal commercial use.
*   **For Competitors:** You may not offer CloaKey as a managed service or incorporate it into a proprietary competing developer utility product.

See [LICENSE](LICENSE) for the full text.

We welcome community contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) to set up your local development environment and learn how to submit a pull request.

---

<p align="center">
  <b>Protect the workflow. Don't stop the workflow.</b><br>
  Made with 🦀 for developers, parents, and pet owners.
</p>
