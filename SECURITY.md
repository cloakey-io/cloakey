# Security Policy

We take the security and privacy of CloaKey seriously. Because our application interacts directly with operating system hooks and intercepts input devices, maintaining the integrity of our software is essential.

## Supported Versions

Only the latest release version of CloaKey is supported for security updates. We recommend all users upgrade immediately if a security release is published.

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0.0 | :x:                |

---

## Reporting a Vulnerability

If you discover a security vulnerability in CloaKey, **please do not open a public GitHub issue**. Instead, report it privately to our team so we can address it responsibly.

### Steps to Report
1.  Send an email to **hello@cloakey.io** with the subject line `SECURITY VULNERABILITY: [Brief description]`.
2.  Provide a detailed description of the vulnerability, including:
    -   The version of CloaKey affected.
    -   Steps to reproduce the vulnerability (including proof-of-concept scripts or commands, if available).
    -   The potential impact of the vulnerability.
3.  If you wish to encrypt your communication, request our PGP public key in your initial message.

### Our Commitment
We will acknowledge receipt of your vulnerability report within **48 hours** and provide a preliminary response. We commit to keeping you updated as we investigate, remediate, and prepare a coordinated disclosure.

---

## Audit Workflows
To protect our users from supply chain attacks, CloaKey uses the following automated security checks on every pull request:
-   `cargo audit` to scan dependencies for known vulnerabilities.
-   Dependency lockfile tracking to detect unauthorized changes.
-   GitHub Actions environments pinned to trusted actions.
