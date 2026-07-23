# Security Policy (SECURITY.md) - MyNMON

**English** | [日本語版](../ja/SECURITY.md)

This document outlines the security policies and vulnerability reporting procedures for the `MyNMON` project.

---

## 1. Security Design and Safety

`MyNMON` ensures high security and reliability through the following design principles:

1. **Memory Safety**:
   - Leveraging Rust's powerful ownership system and type safety, the application eliminates common memory safety vulnerabilities such as buffer overflows and null pointer dereferences.
2. **Local-Only Operation**:
   - While the monitor tracks network interface I/O speeds, it does not make any outbound network connections or transmit data to external servers. It operates strictly locally to collect system metrics and display them in the terminal.
3. **Safe Shutdown**:
   - On termination via signals or keystrokes, cleanup routines are executed to restore the terminal's raw mode and settings to their original state.

---

## 2. Supported Versions

Security updates are provided for the following versions:

| Version | Status |
| :--- | :---: |
| Latest Release (`v0.5.x` and later) | ✅ Supported |
| Older Releases | ❌ Unsupported |

---

## 3. Reporting a Vulnerability

If you discover a potential security vulnerability in `MyNMON`, please do not open a public issue. Instead, follow these steps to report it:

1. **Reporting Point of Contact**:
   - Contact the repository maintainer directly or email the security contact.
2. **Details to Include**:
   - The affected version of `MyNMON` and the OS environment.
   - A detailed description of the vulnerability and steps to reproduce it (proof-of-concept code or commands).
3. **Response Process**:
   - Upon receiving the report, we will acknowledge it within 3 days, prepare and verify a security patch, and publish a release as soon as possible.
