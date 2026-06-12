# Package Distribution Strategy

This document outlines the distribution setup, configurations, and versioning scripts for compiling and publishing CloaKey packages across Windows, macOS, Linux, and developer package systems.

---

## 🪟 Windows Package Managers

Windows is CloaKey's primary release target. We automate package distribution to keep client systems updated.

### 1. Windows Package Manager (Winget)
To distribute CloaKey via `winget`, we submit manifests to the [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) repository.

#### Draft Winget Manifest (`CloaKey.CloaKey.yaml`):
```yaml
# yaml-language-server: $schema=https://github.com/microsoft/winget-cli/releases/download/v1.6.3133/manifest.singleton.1.6.0.schema.json
PackageIdentifier: CloaKey.CloaKey
PackageVersion: 1.0.0
PackageName: CloaKey
Publisher: CloaKey Project
License: BSL-1.1
ShortDescription: Protect the workflow. Don't stop the workflow.
Description: Temporarily lock keyboard and mouse input without interrupting active background processes or screen visibility.
Moniker: cloakey
Tags:
  - keyboard-lock
  - mouse-lock
  - utility
  - workflow
  - productivity
Installers:
  - Architecture: x64
    InstallerType: nullsoft
    InstallerUrl: https://github.com/cloakey/cloakey/releases/download/v1.0.0/cloakey-installer.exe
    InstallerSha256: <SHA256_HASH_OF_RELEASE_EXE>
    UpgradeBehavior: install
ManifestType: singleton
ManifestVersion: 1.6.0
```

### 2. Scoop
Scoop is a popular command-line installer for Windows developer toolsets. We distribute CloaKey via a custom bucket or by submitting to the Main bucket.

#### Draft Scoop Manifest (`cloakey.json`):
```json
{
  "version": "1.0.0",
  "description": "Protect the workflow. Don't stop the workflow.",
  "homepage": "https://cloakey.io",
  "license": "BSL-1.1",
  "architecture": {
    "64bit": {
      "url": "https://github.com/cloakey/cloakey/releases/download/v1.0.0/cloakey-windows-x64.zip",
      "hash": "<SHA256_HASH>",
      "bin": ["cloak.exe", "cloak-tray.exe"]
    }
  },
  "shortcuts": [
    ["cloak-tray.exe", "CloaKey"]
  ],
  "checkver": {
    "github": "https://github.com/cloakey/cloakey"
  },
  "autoupdate": {
    "architecture": {
      "64bit": {
        "url": "https://github.com/cloakey/cloakey/releases/download/v$version/cloakey-windows-x64.zip"
      }
    }
  }
}
```

### 3. Chocolatey
Chocolatey is used for corporate systems management. We publish `cloakey.nupkg` packages to the community gallery.

---

## 🦀 Cargo (Rust Crate Ecosystem)

Developers can install CloaKey via Cargo:
```bash
cargo install cloakey
```

### Publishing Steps
1.  Verify workspace lints:
    ```bash
    cargo clippy --workspace --all-targets -- -D warnings
    ```
2.  Perform a dry-run check:
    ```bash
    cargo publish --dry-run
    ```
3.  Publish crates in order of dependency:
    -   `crates/cloakey-settings`
    -   `crates/cloakey-core`
    -   `crates/cloakey-input`
    -   `crates/cloakey-overlay`
    -   (Finally) the CLI execution crate: `crates/cloakey-cli`
    ```bash
    cargo publish -p cloakey-settings
    # Wait for synchronization...
    cargo publish -p cloakey-core
    # ...
    ```

---

## 🍏 macOS (Homebrew Formula)

Planned for V3. We will submit a formula to the Homebrew core tap.

#### Draft Homebrew Formula (`cloakey.rb`):
```ruby
class Cloakey < Formula
  desc "Lightweight workflow protection utility for mouse and keyboard"
  homepage "https://cloakey.io"
  url "https://github.com/cloakey/cloakey/releases/download/v1.0.0/cloakey-macos.zip"
  sha256 "<SHA256_HASH>"
  license "BSL-1.1"

  def install
    bin.install "cloak"
  end

  test do
    system "#{bin}/cloak", "status"
  end
end
```
