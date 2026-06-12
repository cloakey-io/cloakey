# CloaKey Project Blueprint

Version: 1.0
Status: Foundational Blueprint
Product Type: Desktop Utility / Workflow Protection Platform

---

# Executive Summary

CloaKey is a lightweight desktop utility designed to protect active workflows from accidental human, pet, child, or environmental interaction without interrupting system execution.

Unlike traditional operating system locking mechanisms that hide or suspend active work, CloaKey keeps the workspace visible and running while selectively suppressing keyboard and mouse interaction.

The long-term vision is to create the world's first dedicated Human Input Protection Platform.

---

# Vision

Protect workflows without stopping them.

CloaKey exists to solve a gap that modern operating systems largely ignore:

Accidental interaction.

Whether caused by:

* Users
* Children
* Pets
* Coworkers
* AI workflow interruption
* Accessibility challenges

CloaKey creates a temporary protection layer between input devices and the operating system.

---

# Mission

Build the simplest, safest, and most reliable workflow protection system available for desktop computing.

Core objectives:

* Prevent accidental input
* Preserve workflow continuity
* Protect AI-assisted workflows
* Support productivity and focus
* Remain lightweight and trustworthy

---

# Product Philosophy

Traditional Locking:

Visible: No
Workflow Running: Maybe
User Access: No

CloaKey Locking:

Visible: Yes
Workflow Running: Yes
User Access: Controlled

The product should always feel:

* Lightweight
* Fast
* Predictable
* Safe
* Non-intrusive

---

# Core Problem Statement

Computers provide:

* Security
* Authentication
* Permissions
* Process isolation

But computers do not provide:

* Temporary workflow protection
* Interaction freezing
* Input isolation
* AI execution protection

Common issues:

* Accidental Ctrl+Z
* Unwanted keyboard presses
* Cat walking on keyboard
* Toddler interaction
* Stream interruptions
* Presentation disruptions
* Terminal mistakes
* Workflow corruption

---

# Product Scope

CloaKey is not:

* A password manager
* A parental control suite
* A screen locker
* A monitoring application
* Surveillance software

CloaKey is:

A workflow interaction protection tool.

---

# Target Users

Primary Users

* Developers
* AI-assisted coders
* System administrators
* Power users
* Streamers
* Designers
* Video editors

Secondary Users

* Parents
* Pet owners
* Educators
* Office workers

Future Users

* Enterprises
* Trading firms
* Industrial operators
* Healthcare systems

---

# Product Roadmap

## V1 - Core Utility

Objective:

Prove utility and establish trust.

Features:

* Keyboard lock
* Mouse lock
* Full lock
* Ghost mode
* Timed lock
* Terminal interface
* Emergency unlock
* Overlay system
* Installation via package managers

Supported Platforms:

* Windows

Installation:

winget install cloakey

Launch:

cloak

or

cloakey

---

## V2 - Experience Layer

Objective:

Make protection enjoyable and visible.

New Features:

* Pet mode
* Child mode
* Floating key visualizer
* Mouse movement mode
* Interactive overlays
* Input absorption system
* Small keyboard-based microgames
* Enhanced profile system

Potential Games:

* Keyboard piano
* Bubble pop
* Cat toy mode
* Reaction games

All games must run inside the protection layer and never affect the actual operating system.

---

## V3 - Professional Edition

Objective:

Advanced workflow control.

Features:

* Desktop-level locking
* Window-level locking
* Application-specific protection
* Profiles
* Automation rules
* AI workflow integration
* Workspace protection templates

Profiles:

* Coding
* AI
* Streaming
* Presentation
* Pet
* Child

---

## V4 - Enterprise Platform

Objective:

Commercial expansion.

Features:

* Centralized management
* Policy enforcement
* Audit logs
* Remote administration
* Organization deployment
* Team controls

Target Markets:

* Trading firms
* Healthcare
* Broadcast studios
* Education
* Manufacturing

---

# User Experience

First Run

User installs:

winget install cloakey

User launches:

cloak

User sees:

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

# Settings

Options:

* Lock shortcut
* Unlock shortcut
* Overlay style
* Overlay size
* Overlay opacity
* Startup behavior
* Notifications

---

# Overlay Modes

None

No visual indication.

Minimal

Small corner logo.

Standard

Center translucent logo.

Prominent

Large visible protection indicator.

Custom

User-defined size and opacity.

---

# Safety Requirements

Critical Rule:

User must never become trapped.

Requirements:

* Multiple unlock methods
* Emergency recovery
* Deadman protection
* Auto unlock option
* Safe mode fallback

Unlock must always take priority.

---

# Security Principles

CloaKey must never:

* Log keystrokes
* Collect passwords
* Monitor user activity
* Store private input
* Upload user data

Input may be blocked.

Input must never be recorded.

---

# Technical Direction

Primary Language:

Rust

Reasons:

* Safety
* Performance
* Small binaries
* Reliability
* Cross-platform future

Future UI:

* Tauri
* Native Windows UI

Avoid:

* Electron

---

# Distribution Strategy

Primary:

* GitHub
* Website

Package Managers:

* Winget
* Homebrew
* Cargo
* Apt (future)

---

# Open Source Strategy

Community Edition:

* Core utility
* CLI
* Basic protection

Commercial Edition:

* Advanced profiles
* Enterprise tools
* Management platform

Trust is essential because the product interacts with system input.

Transparency is a competitive advantage.

---

# Monetization

Free

* Core protection
* Keyboard lock
* Mouse lock
* Ghost mode

Pro

* Profiles
* Automation
* Desktop locking
* Window locking
* AI integrations

Enterprise

* Team management
* Policies
* Audit logs
* Deployment systems

---

# Success Metrics

V1

* GitHub stars
* Downloads
* Active users

V2

* Community growth
* User retention
* Social sharing

V3

* Paid conversions

V4

* Enterprise customers

---

# Long-Term Vision

As AI systems become more autonomous, accidental human interaction becomes more costly.

The future needs:

* Workflow protection
* Input isolation
* AI execution safety
* Human interaction control

CloaKey aims to become the standard protection layer between users and active computing workflows.

---

# Final Principle

Do not stop the workflow.

Protect the workflow.
