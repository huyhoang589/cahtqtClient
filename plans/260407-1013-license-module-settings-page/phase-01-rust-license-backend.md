---
phase: 1
title: "Rust License Backend Modules"
status: done
effort: 3h
completed: 2026-04-07
---

# Phase 1: Rust License Backend Modules

## Context Links
- [Plan](plan.md)
- [2F-HBLS Spec](../../feature/1.%20license/2F_Hardware_Bound_License_CAHTQT_Spec.docx)
- Existing etoken module: `src-tauri/src/etoken/`
- Existing models: `src-tauri/src/etoken/models.rs`

## Overview
- **Priority**: P1 — foundation for all license features
- **Status**: pending
- **Description**: Create `src-tauri/src/license/` module with all backend logic for machine fingerprint collection, license file parsing/verification, challenge-response, and error types.

## Key Insights
- Existing `etoken` module already handles PKCS#11 library loading, slot enumeration, token info, cert reading — reuse `token_manager.rs` functions
- `cryptoki 0.6` already in Cargo.toml — C_Sign for challenge-response available
- `rsa 0.9` + `sha2 0.10` already present — RSA signature verification ready
- Need to add `hex 0.4` and `base64 0.22` to Cargo.toml

## Requirements

### Functional
- Collect CPU Processor ID (wmic on Windows, /proc/cpuinfo on Linux)
- Collect motherboard serial (wmic on Windows, /sys/class/dmi on Linux)
- Compute machine_fp = HEX(SHA-256(cpu_id:board_serial)[0..8]) — 16 hex chars
- Read + parse license.dat from AppData path
- Verify RSA signature over license payload using embedded server public key
- Challenge-response: random nonce → SHA-256(nonce||machine_fp||version) → C_Sign → verify
- Full 11-step verification pipeline returning LicenseState enum

### Non-Functional
- Verification completes < 300ms
- Deterministic fingerprint computation (identical on every call, same hardware)
- No sensitive data (cpu_id, board_serial, private keys) exposed to frontend

## Architecture

```
src-tauri/src/license/
├── mod.rs          — pub fn is_licensed() → LicenseState, re-exports
├── machine.rs      — get_cpu_id(), get_board_serial(), get_machine_fingerprint()
├── token.rs        — challenge-response using PKCS#11 C_Sign (reuses etoken infra)
├── payload.rs      — LicensePayload struct, parse license.dat, verify RSA signature
├── error.rs        — LicenseError enum, LicenseState enum, user-facing messages
```

## Related Code Files

### Files to Create
- `src-tauri/src/license/mod.rs`
- `src-tauri/src/license/machine.rs`
- `src-tauri/src/license/token.rs`
- `src-tauri/src/license/payload.rs`
- `src-tauri/src/license/error.rs`

### Files to Modify
- `src-tauri/Cargo.toml` — add `hex = "0.4"`, `base64 = "0.22"`
- `src-tauri/src/lib.rs` — add `pub mod license;`

### Files to Read for Context
- `src-tauri/src/etoken/token_manager.rs` — PKCS#11 session management patterns
- `src-tauri/src/etoken/models.rs` — TokenInfo, existing types
- `src-tauri/src/commands/settings.rs` — AppState pattern, output_data_dir resolution

## Implementation Steps

### 1. Add Cargo dependencies
Add to `[dependencies]` in `Cargo.toml`:
```toml
hex = "0.4"
base64 = "0.22"
```

### 2. Create `error.rs` — LicenseError + LicenseState enums
```rust
#[derive(Debug, Clone, Serialize)]
pub enum LicenseStatus {
    Valid, Expired, NotFound, NoToken,
    TokenMismatch, MachineMismatch, Corrupted,
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseInfo {
    pub status: LicenseStatus,
    pub expires_at: Option<i64>,  // unix timestamp
    pub product: Option<String>,
}

#[derive(Debug)]
pub enum LicenseError {
    TokenMissing(String),
    InvalidKey(String),
    TokenMismatch,
    MachineMismatch,
    Expired,
    NotLicensed,
    Corrupted(String),
}
```
Include `impl Display` with user-facing messages per spec.

### 3. Create `machine.rs` — hardware ID collection
- `get_cpu_id()` → `Option<String>`: Windows uses `wmic cpu get ProcessorId /value`, Linux uses `/proc/cpuinfo`
- `get_board_serial()` → `Option<String>`: Windows uses `wmic baseboard get SerialNumber /value`, Linux uses `/sys/class/dmi/id/board_serial`
- Reject placeholder values: "To be filled by O.E.M.", "Default string", strings < 4 chars
- `get_machine_fingerprint()` → `String`: SHA-256(cpu_id:board_serial)[0..8] → 16 hex chars
- Fallback to "UNAVAIL" if hardware ID unreadable

### 4. Create `payload.rs` — license file parsing + RSA verification
- `LicensePayload` struct matching spec JSON schema
- `read_license_file(app_data_dir)` → reads `license.dat`, Base64-decodes, splits by `b"||SIG||"`
- `verify_license_signature(payload_bytes, sig_bytes)` → RSA-PKCS1v15-SHA256 using embedded server public key (compile-time constant)
- `parse_license_payload(bytes)` → deserialize JSON to LicensePayload
- Placeholder for server public key PEM (to be provided by admin)

### 5. Create `token.rs` — challenge-response verification
<!-- Updated: Validation Session 1 - Use shared PKCS#11 session from AppState -->
- Accept shared PKCS#11 session handle from caller (passed down from AppState) — do NOT open a new session
- `verify_token_challenge(session, private_key_handle, machine_fp)`:
  - Generate 32-byte random nonce
  - Compute challenge = SHA-256(nonce || machine_fp || CARGO_PKG_VERSION)
  - C_Sign(challenge) on token
  - Verify signature with token's public key from certificate
- Return `Result<(), LicenseError>`

### 6. Create `mod.rs` — orchestration
- `is_licensed(pkcs11_lib_path, app_data_dir)` → `LicenseInfo`:
  - Phase A: Token verification (steps 1-3)
  - Phase B: License binding (steps 4-11)
  - Returns `LicenseInfo { status, expires_at, product }`
- Cache-friendly: designed to be called once at startup, result stored in AppState

## Todo List
- [x] Add hex, base64 to Cargo.toml
- [x] Create license/error.rs with enums + user messages
- [x] Create license/machine.rs with CPU ID + board serial + fingerprint
- [x] Create license/payload.rs with license.dat parsing + RSA verification
- [x] Create license/token.rs with challenge-response
- [x] Create license/mod.rs with is_licensed() pipeline
- [x] Add `pub mod license;` to lib.rs
- [x] Compile check passes

## Success Criteria
- `cargo check` passes with all new modules
- machine.rs produces deterministic fingerprint on same hardware
- payload.rs correctly splits, decodes, and verifies test license data
- All error variants map to correct user-facing messages

## Risk Assessment
- **wmic deprecation**: Windows 11 may deprecate wmic. Mitigation: can switch to PowerShell `Get-CimInstance` later, wmic still works on Windows 10/11 currently
- **Server public key**: Need actual PEM from admin. Use placeholder for development.
- **PKCS#11 session reuse**: Challenge-response needs an open session. Must coordinate with existing etoken module to avoid conflicting sessions.

## Security Considerations
- Private key NEVER leaves token hardware (C_Sign happens on chip)
- Server public key embedded at compile time — cannot be swapped at runtime
- machine_fp computation uses SHA-256 — deterministic, collision-resistant
- No sensitive hardware IDs (cpu_id, board_serial) exposed to frontend
- Nonce is random per launch — defeats binary patching attacks
