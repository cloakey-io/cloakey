# Changelog

All notable changes to CloaKey will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

## [1.0.0] - 2026-06-10

### Added
- **Keyboard Lock** — Block all keyboard input while applications continue running
- **Mouse Lock** — Block all mouse input (movement, clicks, scroll) while applications continue running
- **Full Lock** — Block both keyboard and mouse simultaneously
- **Ghost Mode** — Mouse movement allowed, all clicks and keyboard input blocked
- **Timed Lock** — Auto-unlock after a specified duration (supports seconds, minutes, hours)
- **Emergency Unlock** — CTRL + ALT + SHIFT held for 2 seconds; works in all modes and cannot be disabled
- **Status Command** — `cloak status` shows current state, what is blocked, remaining timer, and unlock shortcut
- **Settings Menu** — Configure lock shortcuts, unlock shortcut, overlay mode
- **Startup Integration** — Configure CloaKey to run on Windows startup via Windows Registry
- **Help Menu** — `cloak help` lists all commands with examples
- **About** — `cloak about` shows version, license, website, and GitHub URLs
- **Interactive Menu** — ratatui-based terminal UI with keyboard navigation (no mouse required)
- **Direct Commands** — All features accessible via command-line arguments (scriptable)
- **Overlay System** — Optional always-on-top status display (None, Minimal, Standard modes)
- **TOML Configuration** — User config stored at `%APPDATA%\CloaKey\config.toml`

### Security
- Input is blocked but NEVER recorded, saved, or transmitted
- No telemetry, no analytics, no user accounts
- Emergency unlock always takes priority over all lock states

### Performance
- Startup time: < 1 second
- Idle CPU: < 1%
- Memory usage: < 50 MB
- Input hook latency: < 50ms
- Unlock response time: < 100ms

[Unreleased]: https://github.com/cloakey/cloakey/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/cloakey/cloakey/releases/tag/v1.0.0
