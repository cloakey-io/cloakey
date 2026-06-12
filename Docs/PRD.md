# PRD.md

# CloaKey Product Requirements Document (PRD)

Version: 1.0

Status: Approved for Development

Product: CloaKey

Category: Desktop Workflow Protection Utility

Target Release: V1 MVP

---

# Executive Summary

CloaKey is a lightweight desktop utility that protects active workflows from accidental interaction without interrupting system execution.

Users can temporarily disable keyboard input, mouse input, or both while keeping applications, videos, downloads, AI agents, terminals, and other processes running normally.

The first release focuses on reliability, safety, simplicity, and trust.

The product must feel instant, predictable, and impossible to accidentally misuse.

---

# Problem Statement

Desktop operating systems currently provide only two extremes:

1. Full interaction
2. Full lock

There is no simple middle ground where users can:

* Keep their workflow visible
* Keep applications running
* Prevent accidental input

This creates problems such as:

* Accidental keyboard presses
* Accidental shortcuts
* Interrupted coding sessions
* Pet interference
* Child interference
* Terminal mistakes
* Presentation disruptions
* AI workflow interruption

CloaKey fills this gap.

---

# Product Vision

Create a workflow protection layer that allows users to temporarily suppress interaction without interrupting execution.

Core principle:

Protect the workflow.
Do not stop the workflow.

---

# Objectives

## Primary Objectives

* Prevent accidental keyboard input
* Prevent accidental mouse input
* Maintain application execution
* Preserve desktop visibility
* Ensure reliable unlocking

---

## Secondary Objectives

* Build user trust
* Create a lightweight utility
* Establish foundation for future expansion

---

# Success Metrics

## Technical

Keyboard blocking success rate:

99.99%

Mouse blocking success rate:

99.99%

Unlock success rate:

100%

Crash rate:

< 0.1%

Startup time:

< 1 second

Memory usage:

< 50 MB

CPU usage when idle:

< 1%

---

## Product

GitHub Stars:

1,000+

Downloads:

5,000+

Issue Resolution:

< 7 days

Positive User Feedback:

80%+

---

# User Personas

## Persona 1: Developer

Needs:

* Protect coding sessions
* Prevent accidental edits
* Prevent terminal mistakes

---

## Persona 2: AI User

Needs:

* Protect AI execution
* Avoid accidental interruption

---

## Persona 3: Pet Owner

Needs:

* Prevent cat keyboard attacks
* Prevent accidental clicks

---

## Persona 4: Parent

Needs:

* Temporary protection from child interaction

---

## Persona 5: Presenter

Needs:

* Protect presentation workflow
* Prevent accidental shortcuts

---

# User Stories

## Keyboard Lock

As a user,

I want to disable keyboard input,

So that accidental key presses do not affect my workflow.

---

## Mouse Lock

As a user,

I want to disable mouse input,

So that accidental clicks do not affect my workflow.

---

## Full Lock

As a user,

I want to disable both keyboard and mouse input,

So that my workflow remains protected.

---

## Ghost Mode

As a user,

I want the mouse to move without clicking,

So I can observe my desktop safely.

---

## Timed Lock

As a user,

I want CloaKey to unlock automatically,

So I don't forget to unlock it.

---

## Emergency Unlock

As a user,

I need a guaranteed unlock method,

So I never become trapped.

---

# MVP Scope

# Included Features

## Keyboard Lock

Description:

Blocks keyboard input system-wide.

Requirements:

* Blocks all standard keys
* Blocks shortcuts
* Blocks function keys
* Does not affect unlock sequence

---

## Mouse Lock

Description:

Blocks mouse interaction.

Requirements:

* Blocks clicks
* Blocks dragging
* Blocks scroll wheel

Optional:

* Allow cursor movement

---

## Full Lock

Description:

Blocks keyboard and mouse simultaneously.

Requirements:

* Applications continue running
* Screen remains visible

---

## Ghost Mode

Description:

Allows mouse movement but suppresses interaction.

Requirements:

* Cursor visible
* No click execution
* No drag execution
* Keyboard disabled

---

## Timed Lock

Description:

Automatically unlocks after specified duration.

Examples:

30 seconds

60 seconds

5 minutes

10 minutes

---

## Status Command

Description:

Display current lock state.

Must show:

* Active mode
* Remaining timer
* Unlock shortcut

---

## Emergency Unlock

Description:

Guaranteed recovery mechanism.

Default:

CTRL + ALT + SHIFT (hold 2 seconds)

Requirements:

* Must work during all modes
* Must override all locks

---

## Settings Menu

Must include:

* Lock shortcut
* Unlock shortcut
* Overlay mode
* Logo size
* Logo opacity

---

## Help Menu

Must include:

* Commands
* Examples
* Troubleshooting
* Website URL
* GitHub URL

---

## About Menu

Must include:

* Version
* License
* Website
* GitHub

---

# CLI Requirements

Command:

cloak

Alternative:

cloakey

---

# Interactive Launch

Running:

cloak

Opens:

=================================
CloaKey
Workflow Protection System
==========================

1. Lock Keyboard
2. Lock Mouse
3. Lock Both
4. Ghost Mode
5. Timed Lock
6. Settings
7. Help
8. About

Q. Exit

---

# Direct Commands

cloak keyboard

cloak mouse

cloak lock

cloak ghost

cloak unlock

cloak timer 60

cloak status

---

# Overlay Requirements

Modes:

None

Minimal

Standard

Prominent

Custom

---

# Overlay Elements

Optional:

* Logo
* Lock icon
* Timer
* Status text

---

# Non-Functional Requirements

## Performance

Launch time:

< 1 second

Input interception latency:

< 50ms

---

## Reliability

No system crashes.

No frozen desktop.

No permanent lock states.

---

## Security

Must not:

* Record keystrokes
* Save passwords
* Monitor user activity
* Upload user data

---

## Privacy

No telemetry in MVP.

No analytics in MVP.

No account system.

No login system.

---

# Out of Scope (V1)

The following are NOT part of V1:

* Desktop locking
* Window locking
* Enterprise features
* Cloud accounts
* User accounts
* AI integrations
* Remote management
* Team features
* Pet mode
* Child mode
* Games
* Automation rules

These belong in later releases.

---

# V2 Planned Features

* Pet Mode
* Child Mode
* Floating Key Visualizer
* Input Absorption Layer
* Keyboard Piano
* Bubble Pop
* Microgames
* Enhanced Overlay System

---

# V3 Planned Features

* Desktop-Level Locking
* Window-Level Locking
* Application Profiles
* Automation Rules
* AI Workflow Protection

---

# V4 Planned Features

* Enterprise Dashboard
* Policy Management
* Audit Logs
* Team Administration
* Remote Control

---

# Acceptance Criteria

A release is considered complete when:

✓ Keyboard lock works

✓ Mouse lock works

✓ Full lock works

✓ Ghost mode works

✓ Timed lock works

✓ Emergency unlock works

✓ No critical bugs remain

✓ Installation works through package managers

✓ Documentation is complete

✓ GitHub release published

---

# Final Product Principle

Users should be able to walk away from an active workflow for 30 seconds, 5 minutes, or 30 minutes and return knowing nothing accidental modified their work.

CloaKey exists to provide that confidence.
