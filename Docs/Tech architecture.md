# Technical Architecture.md

Version: 1.0

Product: CloaKey

Status: Approved Architecture

Primary Language: Rust

Target Platform: Windows (Initial Release)

Architecture Style: Modular Monolith

---

# Purpose

This document defines the technical architecture, module boundaries, system design principles, and implementation approach for CloaKey.

Goals:

* Reliability
* Safety
* Performance
* Low resource usage
* Future scalability
* Cross-platform readiness

---

# Architectural Principles

## Principle 1

Safety before features.

The system must never trap users.

Emergency unlock always takes priority.

---

## Principle 2

Input suppression only.

CloaKey blocks input.

CloaKey does not record input.

---

## Principle 3

Keep the core lightweight.

Avoid unnecessary frameworks.

Avoid Electron.

Avoid large runtime dependencies.

---

## Principle 4

Core logic must remain platform-independent whenever possible.

Only platform adapters should contain OS-specific code.

---

# Technology Stack

## Core Language

Rust

Reasons:

* Memory safety
* High performance
* Low memory footprint
* Strong concurrency support
* Excellent CLI ecosystem
* Future cross-platform support

---

## CLI Framework

clap

Purpose:

* Argument parsing
* Command handling
* Help generation

---

## Terminal UI

ratatui

Purpose:

* BIOS-style interface
* Keyboard navigation
* Lightweight UI

---

## Serialization

serde

Formats:

* JSON
* TOML

---

## Configuration

toml

Location:

User configuration directory

Example:

config.toml

---

## Logging

tracing

Purpose:

* Debugging
* Diagnostics

No telemetry collection.

---

# High-Level Architecture

+----------------------+
| CLI Layer |
+----------------------+
|
v
+----------------------+
| Application Core |
+----------------------+
|
+------------------+
| |
v v

+----------+ +----------+
| Input | | Session |
| Engine | | Manager |
+----------+ +----------+
|
v

+----------------------+
| Lock State Engine |
+----------------------+
|
v

+----------------------+
| Overlay System |
+----------------------+
|
v

+----------------------+
| Safety Controller |
+----------------------+

---

# Repository Structure

cloakey/

├── apps/
│
├── cli/
│
├── core/
│
├── input/
│
├── locks/
│
├── overlays/
│
├── profiles/
│
├── settings/
│
├── platform/
│
├── integrations/
│
├── docs/
│
├── scripts/
│
├── tests/
│
└── examples/

---

# Core Modules

## CLI Module

Responsibilities:

* Parse commands
* Display menus
* Handle user selections
* Launch lock modes

Examples:

cloak lock

cloak keyboard

cloak mouse

cloak ghost

cloak status

---

## Core Module

Purpose:

Business logic.

Responsibilities:

* Lock orchestration
* State transitions
* Profile execution
* Timer management

No OS-specific code.

---

## Input Engine

Purpose:

Intercept keyboard and mouse events.

Responsibilities:

* Keyboard suppression
* Mouse suppression
* Event filtering

Never:

* Save keys
* Record keys
* Store passwords

---

# Windows Input Layer

Uses:

SetWindowsHookEx

WH_KEYBOARD_LL

WH_MOUSE_LL

Purpose:

Global keyboard hook.

Global mouse hook.

---

# Event Pipeline

Input Event

↓

Input Engine

↓

Lock Evaluation

↓

Allow

or

Block

↓

Operating System

---

# Lock State Engine

Purpose:

Determine whether input should be allowed.

States:

Unlocked

KeyboardLocked

MouseLocked

FullyLocked

GhostMode

TimedLock

Future:

PetMode

ChildMode

DesktopMode

WindowMode

---

# State Machine

Unlocked

↓

KeyboardLock

↓

Unlock

↓

Unlocked

Each state transition must be explicit.

No hidden transitions.

---

# Session Manager

Purpose:

Track active CloaKey session.

Responsibilities:

* Current lock mode
* Lock timer
* Overlay state
* Active profile

---

# Timer Engine

Purpose:

Handle timed locks.

Example:

cloak timer 60

Workflow:

Create timer

↓

Countdown

↓

Auto Unlock

↓

Notification

---

# Overlay System

Purpose:

Provide visual feedback.

Types:

None

Minimal

Standard

Prominent

Custom

---

# Overlay Components

Logo

Status Text

Timer

Lock State

Notifications

---

# Rendering Strategy

Layered window

Always on top

Click-through

Minimal GPU usage

---

# Safety Controller

Most critical component.

Purpose:

Prevent lockouts.

Responsibilities:

* Emergency unlock
* Recovery mode
* Timeout protection
* Deadman switch

---

# Emergency Unlock

Default:

CTRL + ALT + SHIFT

Hold:

2 seconds

Requirements:

Must work in every mode.

Must bypass all locks.

Must never be disabled.

---

# Recovery Mode

Purpose:

Recover from failures.

Example:

Input engine crash.

Workflow:

Detect failure

↓

Disable locks

↓

Return control

---

# Settings System

Location

Windows:

%APPDATA%/CloaKey/

Files:

config.toml

shortcuts.toml

profiles.toml

---

# Config Example

[general]

overlay = "standard"

unlock_timeout = 2

show_logo = true

logo_opacity = 60

---

# Shortcut System

Configurable.

Examples:

lock_all

lock_keyboard

lock_mouse

ghost_mode

unlock

---

# Default Shortcuts

CTRL + ALT + L

Lock All

CTRL + ALT + K

Keyboard Lock

CTRL + ALT + M

Mouse Lock

CTRL + ALT + G

Ghost Mode

CTRL + ALT + SHIFT

Unlock

---

# Package Architecture

V1 Packages

cloakey-cli

cloakey-core

cloakey-input

cloakey-overlay

cloakey-settings

---

# Future Packages

cloakey-desktop

cloakey-window

cloakey-ai

cloakey-enterprise

---

# Security Model

CloaKey is not:

* Spyware
* Monitoring software
* Keylogger

Principles:

No input storage

No cloud upload

No telemetry

No analytics

No accounts

No tracking

---

# Antivirus Considerations

Potential triggers:

Input hooks

Global hooks

Suppression behavior

Mitigation:

Code signing

Open source core

Transparent codebase

Clear documentation

---

# Performance Targets

Memory

Under 50MB

CPU

Under 1%

Startup

Under 1 second

Hook latency

Under 50ms

Unlock response

Under 100ms

---

# Testing Strategy

Unit Tests

90%+ coverage

Integration Tests

All lock modes

All shortcuts

All timers

Mutation Testing

Required

---

# Reliability Testing

Required scenarios:

Keyboard spam

Mouse spam

Rapid lock/unlock

Long runtime

System sleep/wake

Multi-monitor

Remote desktop

Virtual machine

Pet simulation

Child simulation

---

# Future Architecture

V2

Input Absorption Layer

Visual Key Effects

Microgames

Pet Mode

Child Mode

---

# V3

Desktop Isolation

Window Isolation

AI Workflow Protection

Automation Rules

Profiles

---

# V4

Enterprise Platform

Remote Management

Audit Logs

Policy Engine

Organization Controls

---

# Deployment Strategy

Initial:

GitHub Releases

Package Managers:

winget

cargo

homebrew

Future:

MSIX

EXE Installer

MS Store

---

# Final Engineering Rule

If there is ever a conflict between:

Feature Convenience

and

User Recovery

User Recovery wins.

A user must always be able to regain control of their computer.
