# CloaKey

<p align="center">
  <img src="../CloaKey logo no bg.png" alt="CloaKey Logo" width="200"/>
</p>

### Protect the workflow. Don't stop the workflow.

CloaKey is a lightweight desktop workflow protection utility that temporarily suppresses keyboard and mouse interaction while allowing applications, AI agents, downloads, videos, terminals, and other processes to continue running normally.

Unlike traditional screen locking systems, CloaKey keeps your desktop visible and active while protecting it from accidental interaction.

---

## Why CloaKey?

Modern operating systems provide two options:

### Fully Interactive

Everything works.

Everything can be accidentally modified.

### Fully Locked

Everything disappears behind a lock screen.

Workflow visibility is lost.

---

CloaKey provides a third option:

### Protected

Your workflow remains visible.

Your applications continue running.

Accidental interaction is blocked.

---

## Common Scenarios

### Developers

Prevent accidental:

* Ctrl + Z
* Ctrl + S
* Terminal interruption
* IDE interaction

### AI Workflows

Protect:

* AI coding sessions
* Long-running agents
* Repository-wide operations
* Automated tasks

### Pet Owners

Protect against:

* Cats walking on keyboards
* Dogs bumping mice
* Curious pets

### Parents

Prevent accidental interaction from:

* Toddlers
* Children
* Visitors

### Presentations

Avoid:

* Accidental shortcuts
* Window switching
* Presentation disruptions

---

# Features

## Keyboard Lock

Disable keyboard input while keeping the mouse active.

---

## Mouse Lock

Disable mouse interaction while keeping the keyboard active.

---

## Full Protection

Disable keyboard and mouse simultaneously.

---

## Ghost Mode

Allow mouse movement.

Block:

* Clicks
* Dragging
* Keyboard input

Perfect for safely observing an active workflow.

---

## Timed Protection

Automatically unlock after a specified duration.

Examples:

```bash
cloak timer 30
cloak timer 5m
cloak timer 1h
```

---

## Emergency Unlock

Every protection mode includes a guaranteed recovery mechanism.

Default:

```text
CTRL + ALT + SHIFT
(Hold 2 Seconds)
```

User recovery always takes priority.

---

# Installation

## Windows

```bash
winget install cloakey
```

---

## Rust / Cargo

```bash
cargo install cloakey
```

---

## macOS

```bash
brew install cloakey
```

(Coming Soon)

---

## Linux

Package support planned.

---

# Quick Start

Launch interactive mode:

```bash
cloak
```

You will see:

```text
=================================
CloaKey
Workflow Protection System
=================================

1. Lock Keyboard
2. Lock Mouse
3. Lock Keyboard & Mouse
4. Ghost Mode
5. Timed Lock
6. Settings
7. Help
8. About

Q. Exit
```

---

# Direct Commands

## Lock Keyboard

```bash
cloak keyboard
```

---

## Lock Mouse

```bash
cloak mouse
```

---

## Lock Everything

```bash
cloak lock
```

---

## Ghost Mode

```bash
cloak ghost
```

---

## Timed Lock

```bash
cloak timer 5m
```

---

## Status

```bash
cloak status
```

---

## Unlock

```bash
cloak unlock
```

---

# Philosophy

CloaKey is built around one principle:

> Protect the workflow. Do not stop the workflow.

Applications continue running.

Examples:

* VS Code
* Cursor
* Claude Code
* Browsers
* Downloads
* Render jobs
* Videos
* Terminal processes

Only interaction changes.

---

# Security & Privacy

CloaKey is not:

* Spyware
* Monitoring software
* Surveillance software
* A keylogger

CloaKey does not:

* Record keystrokes
* Save passwords
* Upload input data
* Collect clipboard contents
* Track user behavior

Input may be blocked.

Input is never recorded.

---

# Performance Goals

Target Metrics:

| Metric         | Goal       |
| -------------- | ---------- |
| Startup Time   | < 1 second |
| Idle CPU       | < 1%       |
| Memory Usage   | < 50MB     |
| Unlock Latency | < 100ms    |
| Input Latency  | < 50ms     |

---

# Roadmap

## V1

* Keyboard Lock
* Mouse Lock
* Full Lock
* Ghost Mode
* Timed Lock
* CLI Interface

---

## V2

* Pet Mode
* Child Mode
* Floating Key Visualizer
* Input Absorption Layer
* Interactive Overlays
* Micro Games

---

## V3

* Desktop Locking
* Window Locking
* Profiles
* Automation Rules
* AI Workflow Protection

---

## V4

* Enterprise Dashboard
* Team Policies
* Remote Administration
* Audit Logging

---

# Building From Source

Clone repository:

```bash
git clone https://github.com/cloakey/cloakey.git
```

Enter project:

```bash
cd cloakey
```

Build:

```bash
cargo build --release
```

Run:

```bash
cargo run
```

---

# Contributing

Contributions are welcome.

Areas needing help:

* Rust Development
* Windows APIs
* Input Handling
* Testing
* Documentation
* Accessibility
* Cross-Platform Support

Please read CONTRIBUTING.md before submitting a pull request.

---

# License

MIT License

See LICENSE for details.

---

# Inspiration

A simple observation:

Sometimes the biggest threat to an active workflow is not malware.

It's a cat sitting on a keyboard.

---

# Final Principle

The future of computing is not only about making machines smarter.

It's also about protecting workflows from unintended interaction.

CloaKey exists to provide that protection.
