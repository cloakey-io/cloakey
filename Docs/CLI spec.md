# CLI Specification.md

**Product:** CloaKey
**Version:** 1.0
**Target Release:** V1 MVP
**Platform:** Windows First
**Interface:** Interactive Terminal Application

---

# Purpose

This document defines the complete command-line interface behavior for CloaKey.

Goals:

* Simple
* Fast
* Discoverable
* Beginner-friendly
* Power-user capable

A user should be able to install CloaKey and understand basic usage within 30 seconds.

---

# CLI Design Principles

## Principle 1

Simple by default.

Most users should only need:

```bash
cloak
```

---

## Principle 2

Interactive first.

The menu should guide users.

---

## Principle 3

Commands remain scriptable.

Power users should be able to bypass menus.

---

## Principle 4

Every action must clearly show:

* Current state
* Current protection mode
* Unlock instructions

---

# Command Names

Primary:

```bash
cloak
```

Alias:

```bash
cloakey
```

Both must behave identically.

---

# Launch Modes

## Interactive Mode

Command:

```bash
cloak
```

Behavior:

Launches the main menu.

---

# Main Menu

```text
=========================================
           CloaKey v1.0
    Workflow Protection System
=========================================

1. Lock Keyboard
2. Lock Mouse
3. Lock Keyboard & Mouse
4. Ghost Mode
5. Timed Lock
6. Settings
7. Help
8. About

Q. Exit

Select:
```

---

# Menu Navigation

Keys:

```text
1-9
Enter
Esc
Q
Arrow Keys
```

Supported.

---

# Invalid Input

Example:

```text
Input: 99
```

Response:

```text
Invalid selection.

Please choose a valid option.
```

Return to menu.

---

# Direct Commands

---

## Lock Keyboard

Command:

```bash
cloak keyboard
```

Alternative:

```bash
cloak lock keyboard
```

Result:

```text
Keyboard Lock Enabled

Keyboard Input: BLOCKED
Mouse Input: ALLOWED

Unlock:
Hold CTRL + ALT + SHIFT
```

---

## Lock Mouse

Command:

```bash
cloak mouse
```

Result:

```text
Mouse Lock Enabled

Mouse Input: BLOCKED
Keyboard Input: ALLOWED

Unlock:
Hold CTRL + ALT + SHIFT
```

---

## Lock Both

Command:

```bash
cloak lock
```

Alternative:

```bash
cloak both
```

Result:

```text
Full Protection Enabled

Keyboard: BLOCKED
Mouse: BLOCKED

Unlock:
Hold CTRL + ALT + SHIFT
```

---

## Ghost Mode

Command:

```bash
cloak ghost
```

Result:

```text
Ghost Mode Enabled

Mouse Movement: ALLOWED
Mouse Clicks: BLOCKED
Keyboard: BLOCKED
```

---

## Timed Lock

Command:

```bash
cloak timer 60
```

Meaning:

60 seconds.

Result:

```text
Protection Enabled

Duration:
60 seconds

Remaining:
60

Unlock:
CTRL + ALT + SHIFT
```

---

# Supported Time Units

Seconds

```bash
cloak timer 30
```

Minutes

```bash
cloak timer 5m
```

Hours

```bash
cloak timer 2h
```

Examples:

```bash
cloak timer 10m
cloak timer 1h
cloak timer 3h
```

---

# Status Command

Command:

```bash
cloak status
```

Output Example:

```text
================================
Current Status
================================

State:
Ghost Mode

Keyboard:
Blocked

Mouse Clicks:
Blocked

Mouse Movement:
Allowed

Remaining Time:
None

Unlock Shortcut:
CTRL + ALT + SHIFT
```

---

# Unlock Command

Command:

```bash
cloak unlock
```

Purpose:

Manual unlock.

Output:

```text
All Protection Disabled

System Restored
```

---

# Settings Command

Command:

```bash
cloak settings
```

Launches:

```text
================================
Settings
================================

1. Shortcuts
2. Overlay
3. Logo
4. Startup
5. Reset

B. Back
```

---

# Shortcut Settings

Menu:

```text
================================
Shortcuts
================================

1. Lock Keyboard
2. Lock Mouse
3. Lock All
4. Ghost Mode
5. Unlock

B. Back
```

---

# Overlay Settings

Menu:

```text
================================
Overlay
================================

1. None
2. Minimal
3. Standard
4. Prominent
5. Custom

B. Back
```

---

# Logo Settings

Menu:

```text
================================
Logo
================================

Show Logo: Yes

Opacity:
60%

Size:
Medium

1. Toggle
2. Opacity
3. Size

B. Back
```

---

# Help Command

Command:

```bash
cloak help
```

Output:

```text
================================
CloaKey Help
================================

Commands:

cloak
cloak keyboard
cloak mouse
cloak lock
cloak ghost
cloak timer
cloak status
cloak unlock

Website:
https://cloakey.io

GitHub:
https://github.com/cloakey/cloakey
```

---

# About Command

Command:

```bash
cloak about
```

Output:

```text
================================
About CloaKey
================================

Version:
1.0

License:
MIT

Website:
https://cloakey.io

GitHub:
https://github.com/cloakey/cloakey
```

---

# Startup Configuration

Command:

```bash
cloak startup
```

Options:

```text
Run On Startup:
Yes / No

Start Minimized:
Yes / No

Load Last Profile:
Yes / No
```

---

# Configuration Commands

---

## Export Config

```bash
cloak export
```

Output:

```text
Configuration exported.
```

---

## Import Config

```bash
cloak import config.toml
```

Output:

```text
Configuration imported.
```

---

## Reset Config

```bash
cloak reset
```

Confirmation:

```text
Reset all settings?

Y/N
```

---

# Error Messages

## Invalid Command

Input:

```bash
cloak banana
```

Output:

```text
Unknown command.

Run:

cloak help
```

---

## Invalid Time

Input:

```bash
cloak timer banana
```

Output:

```text
Invalid duration.

Examples:

cloak timer 60
cloak timer 5m
cloak timer 1h
```

---

# Emergency Unlock Display

Whenever protection is active:

```text
================================

PROTECTION ACTIVE

Unlock:
CTRL + ALT + SHIFT
(Hold 2 Seconds)

================================
```

Must always be visible.

---

# Future Reserved Commands

Not implemented in V1.

Reserved for future releases.

```bash
cloak pet
cloak child
cloak desktop
cloak window
cloak profile
cloak ai
cloak enterprise
cloak policy
cloak remote
```

---

# Exit Behavior

Command:

```bash
Q
```

or

```bash
Esc
```

Result:

```text
Exiting CloaKey...
```

Returns control to terminal.

---

# Accessibility Requirements

Must support:

* Keyboard navigation
* Screen readers
* High contrast terminals
* Large terminal fonts

No mouse required.

---

# Final CLI Principle

A first-time user should be able to:

1. Install CloaKey
2. Run `cloak`
3. Protect a workflow
4. Unlock safely

without reading documentation.

---

## V1 Launch Motto

```text
One command.

One shortcut.

Zero accidents.
```