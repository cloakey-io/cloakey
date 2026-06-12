# Contributing to CloaKey

Thank you for your interest in contributing to CloaKey! We welcome contributions from developers of all skill levels to help protect workflows everywhere.

This document guides you through setting up your workspace, our coding standards, and our pull request submission process.

---

## 🛠️ Getting Started

### Prerequisites
CloaKey is built in Rust. To compile and test the codebase, you need:
- **Rust Toolchain:** Install [Rustup](https://rustup.rs/) (Stable 1.75+ recommended).
- **Windows Build Tools:** CloaKey currently targets Windows systems. Ensure you have the C++ build tools installed via Visual Studio Installer if building natively.
- **Git:** Version control configuration.

### Setup Your Environment
1.  Clone the repository:
    ```bash
    git clone https://github.com/cloakey/cloakey.git
    cd cloakey
    ```
2.  Build the workspace in debug mode:
    ```bash
    cargo build
    ```
3.  Run the tests to ensure everything is functioning correctly:
    ```bash
    cargo test
    ```

---

## 📂 Project Architecture

CloaKey is organized as a Cargo workspace with several crates:
-   [`crates/cloakey-cli`](crates/cloakey-cli): The primary command-line interface entry point.
-   [`crates/cloakey-core`](crates/cloakey-core): Core logic for lock coordination and timers.
-   [`crates/cloakey-input`](crates/cloakey-input): OS-level input hook interception (Windows API hooks).
-   [`crates/cloakey-overlay`](crates/cloakey-overlay): Display overlay components.
-   [`crates/cloakey-settings`](crates/cloakey-settings): User configuration management and TOML parsing.
-   [`crates/cloakey-tray`](crates/cloakey-tray): Windows system tray integration.
-   [`crates/cloakey-installer`](crates/cloakey-installer): Setup helper for Windows installation.

---

## 🎨 Development Workflow

### 1. Branching Model
Always create a new feature branch based off the `main` branch:
```bash
git checkout -b feature/my-amazing-feature
```

### 2. Code Quality & Formatting
We enforce strict style checks. Before committing, run these commands locally:
-   **Formatting:** Check formatting using `rustfmt`:
    ```bash
    cargo fmt --all -- --check
    ```
    To automatically apply formatting changes:
    ```bash
    cargo fmt --all
    ```
-   **Linting:** Analyze code quality using `clippy`:
    ```bash
    cargo clippy --workspace --all-targets -- -D warnings
    ```
-   **Security Auditing:** Check dependencies for known security vulnerabilities:
    ```bash
    cargo audit
    ```
    *(Note: If you do not have cargo-audit installed, install it using `cargo install cargo-audit`)*

### 3. Commit Message Convention
We follow the **Conventional Commits** specification. Commit messages should look like:
```text
<type>(<scope>): <short description>

[Optional longer body explaining why the change was made]

[Optional footer referencing issue numbers, e.g., Closes #123]
```
Allowed types include:
-   `feat`: A new feature (e.g., `feat(input): add support for mouse lock cursor movement toggle`)
-   `fix`: A bug fix (e.g., `fix(core): resolve race condition in timed lock thread`)
-   `docs`: Documentation changes only
-   `style`: Code style tweaks (formatting, semi-colons, etc.)
-   `refactor`: Code changes that neither fix a bug nor add a feature
-   `test`: Adding missing tests or correcting existing tests
-   `chore`: Internal build tool or library updates (e.g., bumping dependencies)

### 4. Developer Certificate of Origin (DCO)
To ensure clear intellectual property ownership, we require all contributions to be signed off. This certifies that you have the right to submit the code under our BSL 1.1 license.
Simply add the `-s` flag to your git commits:
```bash
git commit -s -m "feat(cli): implement interactive settings reset"
```
This appends a line to your commit message:
`Signed-off-by: Your Name <your.email@example.com>`

---

## 🚀 Submitting a Pull Request

1.  Push your changes to your fork or branch.
2.  Open a Pull Request (PR) against the `main` branch of the `cloakey/cloakey` repository.
3.  Fill out the [PULL_REQUEST_TEMPLATE.md](.github/PULL_REQUEST_TEMPLATE.md) completely.
4.  Ensure that all CI checks (build, formatting, clippy, tests) pass.
5.  Maintainers will review your PR. Address feedback by pushing additional commits to your branch.

Once approved, your PR will be merged! Thank you for making CloaKey better.
