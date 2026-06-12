# Platform and Device Compatibility

This document details the support matrix for CloaKey across operating systems, system architectures, and input hardware devices.

---

## 💻 Operating Systems

CloaKey utilizes systems-level API hooks to intercept input devices. Because hook mechanisms vary significantly by operating system, support is developed on a platform-by-platform basis.

| OS Platform | Version / Distribution | Support Status | Notes |
| :--- | :--- | :--- | :--- |
| **Windows** | Windows 10 (Build 19041+) | :white_check_mark: Supported | Fully tested native hooks. |
| **Windows** | Windows 11 | :white_check_mark: Supported | Recommended platform. |
| **Windows Server**| Windows Server 2019 / 2022 | :warning: Limited | Works in active sessions; RDP has hook constraints. |
| **macOS** | macOS 12 Monterey + | :construction: Planned | Scheduled for V3. Requires Accessibility permissions. |
| **Linux** | X11 Environments | :construction: Planned | Scheduled for V3. Requires Xlib hook interfaces. |
| **Linux** | Wayland Environments | :construction: Planned | Scheduled for V3. Requires portal/interception daemons. |

---

## 🏗️ System Architectures

| Architecture | Platform | Support Status | Notes |
| :--- | :--- | :--- | :--- |
| **x86_64** (64-bit Intel/AMD) | Windows | :white_check_mark: Supported | Primary target architecture. |
| **aarch64** (ARM64) | Windows | :warning: Experimental | Compiled via emulator or cross-compilation. |
| **x86** (32-bit Intel/AMD) | Windows | :x: Not Supported | 32-bit Windows is not supported. |

---

## 🖱️ Input Hardware Devices

CloaKey intercepts input at the OS event level, making it compatible with most hardware interfaces.

| Device Type | Support Status | Behavior under Active Lock |
| :--- | :--- | :--- |
| **Standard Keyboard** (USB/Bluetooth) | :white_check_mark: Supported | Intercepted and blocked system-wide. |
| **Standard Mouse** (USB/Bluetooth) | :white_check_mark: Supported | Intercepted (clicks, scrolling, movement blocked). |
| **Laptop Trackpads** (Precision Touch) | :white_check_mark: Supported | Clicks and gestures are blocked; cursor stays in place. |
| **Touchscreens** | :warning: Experimental | Touch gestures may bypass standard hooks on some devices. |
| **Drawing Tablets / Stylus** (Wacom) | :warning: Experimental | Intercepted as absolute mouse input; clicks are blocked. |
| **Virtual Inputs / Macro Tools** | :x: Not Blocked | Inputs generated programmatically (e.g., AutoHotkey) may bypass hooks by design. |
