---
title: "License Module in Settings Page"
description: "Add LICENSE section to Settings with status display, credential export, and license import"
status: done
priority: P1
effort: 8h
branch: feature/license
tags: [license, settings, pkcs11, security, 2f-hbls]
created: 2026-04-07
completed: 2026-04-07
---

# License Module in Settings Page

## Overview

Implement the client-side Two-Factor Hardware-Bound License System (2F-HBLS) for CAHTQT PKI Client. This adds license verification at startup (LicenseGate) and a LICENSE section in Settings for status display, machine credential export, and license import.

## Phases

| # | Phase | Status | Effort | File |
|---|-------|--------|--------|------|
| 1 | Rust License Backend Modules | done | 3h | [phase-01](phase-01-rust-license-backend.md) |
| 2 | Rust Tauri Commands | done | 1.5h | [phase-02](phase-02-rust-tauri-commands.md) |
| 3 | Frontend License Section in Settings | done | 1.5h | [phase-03](phase-03-frontend-license-section.md) |
| 4 | Frontend LicenseGate + Blocking Screens | done | 1.5h | [phase-04](phase-04-frontend-license-gate.md) |
| 5 | Integration + Wiring | done | 0.5h | [phase-05](phase-05-integration-wiring.md) |

## Dependencies

- Existing `etoken` module (PKCS#11 bindings, token_manager)
- Existing `commands/settings.rs` (output_data_dir resolution)
- Cargo dependencies already present: `cryptoki 0.6`, `rsa 0.9`, `sha2 0.10`, `rand 0.8`, `serde_json`, `chrono`, `base64` (needs adding: `0.22`), `hex` (needs adding: `0.4`)

## Key Decisions

1. `get_license_info` works even when license missing — returns status for diagnostics
2. Credential export saves to `output_data_dir` (already configured in app)
3. UI shows ONLY: status badge + expiry date — no fingerprint, no token serial
4. License import via file picker + validation before persisting
5. LicenseGate wraps app root — separate concern from Settings section

## Validation Log

### Session 1 — 2026-04-07
**Trigger:** Initial plan validation before implementation
**Questions asked:** 4

#### Questions & Answers

1. **[UX Flow]** The NoLicenseScreen shows only 'Contact IT' with no action buttons. Should users be able to import a license directly from the blocking screen, or must they get IT to install it?
   - Options: Add Import button (Recommended) | Contact IT only | Import + Export both
   - **Answer:** Import + Export both
   - **Rationale:** Full self-service on the blocking screen — users can export credentials AND import license without needing to reach Settings. Reduces IT support burden.

2. **[Architecture]** PKCS#11 session conflict: both etoken module and license module need PKCS#11 access. How should sessions be coordinated?
   - Options: Shared session in AppState (Recommended) | Separate sessions | Sequential with lock
   - **Answer:** Shared session in AppState
   - **Rationale:** Single PKCS#11 context avoids double-init and slot contention on single-slot tokens. Both modules borrow from AppState.

3. **[Platform]** Should hardware fingerprinting use wmic or PowerShell Get-CimInstance from the start?
   - Options: wmic for now (Recommended) | PowerShell from start | Both with fallback
   - **Answer:** wmic for now
   - **Rationale:** Simpler, well-tested, works on all current targets. YAGNI — switch only if wmic actually breaks.

4. **[Dev Workflow]** Should there be a compile-time dev bypass for the LicenseGate?
   - Options: Yes, cfg(debug) bypass (Recommended) | No bypass | Env var bypass
   - **Answer:** Yes, cfg(debug) bypass
   - **Rationale:** Devs can work without token/license in debug builds. No security risk — compile-time only, stripped from release builds.

#### Confirmed Decisions
- **NoLicenseScreen actions**: Both Export + Import buttons — full self-service workflow
- **PKCS#11 session**: Shared context in AppState, both modules borrow
- **Hardware fingerprinting**: wmic for now, revisit if deprecated
- **Dev bypass**: `cfg(debug_assertions)` auto-pass in LicenseGate

#### Action Items
- [ ] Phase 4: Add Export + Import buttons to NoLicenseScreen
- [ ] Phase 1 & 2: Use shared PKCS#11 context from AppState instead of opening separate sessions
- [ ] Phase 1: Confirm wmic approach (already planned)
- [ ] Phase 4: Add `cfg(debug_assertions)` bypass to LicenseGate and Rust `check_license` command

#### Impact on Phases
- Phase 1: Update token.rs to accept shared PKCS#11 session handle from caller rather than opening its own
- Phase 2: Pass shared PKCS#11 context from AppState to license verification functions; add `cfg(debug_assertions)` bypass to `check_license` command
- Phase 4: NoLicenseScreen gains Export + Import buttons; LicenseGate adds `cfg(debug_assertions)` auto-pass on frontend
