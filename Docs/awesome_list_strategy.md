# Awesome List Inclusion Strategy

To drive organic developer adoption and build trust, CloaKey should be submitted to relevant curated "Awesome Lists" on GitHub. This document outlines the targeting criteria, templates, and requirements for inclusion in these repositories.

---

## 🎯 Target Awesome Lists

| Awesome List | Repository Link | Category Target | Submission Alignment |
| :--- | :--- | :--- | :--- |
| **Awesome Rust** | [rust-unofficial/awesome-rust](https://github.com/rust-unofficial/awesome-rust) | Development tools / Platforms (Windows) | Showcases systems hook integration in safe Rust. |
| **Awesome CLI** | [shadcn/awesome-cli](https://github.com/shadcn/awesome-cli) | Terminal Utilities | Highlights ratatui-based TUI menu and CLI args. |
| **Awesome Windows**| [Awesome-Windows/Awesome](https://github.com/Awesome-Windows/Awesome) | Productivity Utilities | Fills the gap between active desktop and locked OS. |
| **Awesome Productivity**| [jnv/awesome-productivity](https://github.com/jnv/awesome-productivity) | Focus / Safety | Safe shield for coding agents, pets, and children. |

---

## 📋 Pre-Submission Checklist

Before submitting a Pull Request to any Awesome list, ensure CloaKey meets all quality signals:
- [ ] **Repository Hygiene:** Clean README with badging, licensing, contribution guidelines, and code of conduct.
- [ ] **Release Status:** A tagged version (v1.0.0+) exists on GitHub with attached binary release assets.
- [ ] **Clarity:** Description clearly describes the product in under 15 words.
- [ ] **Build Integrity:** CI tests are green.
- [ ] **Alphabetical Sorting:** Entries in awesome lists are almost always sorted alphabetically. Double-check placement.

---

## 📝 Submission PR Templates

### 1. Awesome Rust Submission
- **Target Category:** `Development tools` -> `Platform specifics` or `Utilities`
- **PR Description Proposal:**
  ```markdown
  Add [CloaKey](https://github.com/cloakey/cloakey) - A lightweight Windows utility to temporarily suppress keyboard and mouse input (while keeping screen visible and applications/tasks running). Written in safe Rust.
  ```

### 2. Awesome CLI Submission
- **Target Category:** `Utilities` or `Productivity`
- **PR Description Proposal:**
  ```markdown
  * [CloaKey](https://github.com/cloakey/cloakey) - Protect active terminal runs, AI coding sessions, and videos by blocking accidental keyboard/mouse input without suspending execution. Includes an interactive `ratatui` TUI.
  ```

### 3. Awesome Windows Submission
- **Target Category:** `Utilities` -> `System` or `Productivity`
- **PR Description Proposal:**
  ```markdown
  - [CloaKey](https://github.com/cloakey/cloakey) - A seatbelt for Windows workflows. Suppress keyboard/mouse hooks temporarily to protect active processes from cats, children, and accidental key slips.
  ```
