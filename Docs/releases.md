# Release Strategy & Templates

This document details the versioning rules, tagging workflows, and release notes templates used to publish CloaKey releases.

---

## 🏷️ Versioning Strategy

CloaKey adheres to **Semantic Versioning 2.0.0 (SemVer)**:
-   **MAJOR (`X.0.0`):** Changes that break backward compatibility or represent key project shifts (e.g., transitioning OS hooks api interfaces or introducing major architectural dependencies).
-   **MINOR (`0.Y.0`):** Backward-compatible new features (e.g., adding a new input-hook mode, adding startup configuration arguments).
-   **PATCH (`0.0.Z`):** Backward-compatible bug fixes, optimizations, or documentation adjustments.

---

## 🚀 Release Process Workflow

```text
[Code Freeze] ──> [Bump Version & CHANGELOG] ──> [Create Tag vX.Y.Z] ──> [CI/CD Build] ──> [Publish Release]
```

1.  **Preparation:** Check that all tests are green and clippy has no warnings:
    ```bash
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    ```
2.  **Version Bump:** Update crate version strings in:
    -   `Cargo.toml` (workspace package version)
    -   `CHANGELOG.md` (move unreleased entries into a versioned section header)
3.  **Tagging:** Create a signed git tag:
    ```bash
    git tag -s v1.0.0 -m "Release v1.0.0"
    git push origin v1.0.0
    ```
4.  **Verification:** The `release.yml` GitHub action will compile Windows executables and draft a new GitHub Release automatically.
5.  **Publishing:** Navigate to the releases tab on GitHub, copy the appropriate Release Notes template below, add the version summary, and publish the release.

---

## 📝 Release Notes Templates

### 1. Major Release Template (`v1.x.x`)
```markdown
# Release v1.0.0 — Protected Visibility for Windows Workflows 🔒

We are proud to announce the release of **CloaKey v1.0.0**, establishing the core desktop protection layers for Windows environments.

## 🚀 Key Highlights
- **Keyboard & Mouse Hooks:** Block accidental keyboard inputs and mouse clicks while keeping background tasks visible and active.
- **Interactive TUI:** An interactive, keyboard-navigable ratatui terminal menu (`cloak`).
- **Timed Lock:** Auto-unlock functionality (`cloak timer 5m`) prevents getting locked out.
- **Emergency Override:** Unconditional override sequence (Hold `Ctrl + Alt + Shift` for 2 seconds) restores control instantly.

## 📦 Installation
```powershell
winget install cloakey
```

## 🛠️ What's New
### Crates Included
- `cloakey-cli v1.0.0`
- `cloakey-input v1.0.0`
- `cloakey-core v1.0.0`
- `cloakey-settings v1.0.0`

## 👨‍💻 Contributors
Thank you to everyone who contributed to this release!
- @developer-username
```

### 2. Minor/Patch Release Template (`vX.Y.Z`)
```markdown
# Release v1.1.0 ⚡

This release adds support for [Brief summary of changes, e.g., customizable logo settings] and resolves several minor input hook bugs on Windows 10.

## 🚀 Enhancements
- Added `--minimize` CLI flag to launch CloaKey directly to the system tray.
- Customizable logo opacity controls in TOML config files.

## 🐛 Bug Fixes
- Fixed hook Proc hanging issues on double-monitor setups when cursor transitions screens.
- Resolved configuration folder permission check warnings during startup.

## 📦 Installation
```powershell
winget upgrade cloakey
```
```
