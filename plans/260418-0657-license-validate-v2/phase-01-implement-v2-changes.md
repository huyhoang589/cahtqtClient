---
title: "Implement License v2 Changes"
status: completed
priority: P1
completed_date: 2026-04-18
---

# Phase 01 — Implement v2 Changes

## Context Links
- Plan: [plan.md](plan.md)
- machine.rs: `src-tauri/src/license/machine.rs`
- payload.rs: `src-tauri/src/license/payload.rs`
- mod.rs: `src-tauri/src/license/mod.rs`

## Overview
- **Date:** 2026-04-18
- **Priority:** P1
- **Status:** Pending

Align client license module with server v2 spec. Token serial is now baked into the machine fingerprint (removing the need for a separate `token_serial` field in the payload). Payload struct updated to match server-side schema.

## Key Insights
- `token_serial` is retrieved at `mod.rs:57` before `get_machine_fingerprint()` is called at `mod.rs:60` — order is already correct for passing as arg
- Removing `token_serial` from payload means the separate check at `mod.rs:147-151` becomes dead code and must be deleted
- `machine_fp` becoming required (non-Option) simplifies the binding check — direct equality, no `if let Some(...)` wrapper
- `issued_by` is a new required field — just needs to deserialize cleanly, no validation logic

## Requirements
- `get_machine_fingerprint(token_serial: &str) -> String` hashes `cpu_id:board_serial:token_serial`
- `LicensePayload` matches server-side struct exactly
- All call sites in `mod.rs` updated to compile cleanly
- Cargo build passes with no errors or warnings

## Architecture

```
verify_full()
  └── token_serial = get_token_serial()       ← unchanged
  └── machine_fp = get_machine_fingerprint(token_serial)  ← NEW: pass serial
  └── parse payload → LicensePayload v2       ← new struct, no token_serial field
  └── check machine_fp == license.machine_fp  ← direct String compare (not Option)
  └── [REMOVED] token_serial check            ← baked into fingerprint now
  └── check expiry                            ← unchanged
```

## Related Code Files
- **Modify:** `src-tauri/src/license/machine.rs`
- **Modify:** `src-tauri/src/license/payload.rs`
- **Modify:** `src-tauri/src/license/mod.rs`

## Implementation Steps

### Step 0 — Grep verification (pre-impl)
<!-- Updated: Validation Session 1 - confirm LicensePayload scope -->
```bash
grep -rn "LicensePayload" src-tauri/src/
```
Confirm only `machine.rs`, `payload.rs`, `mod.rs` reference it. If other files appear, update them too before proceeding.

### Step 1 — machine.rs
1. Change `get_machine_fingerprint()` to `get_machine_fingerprint(token_serial: &str) -> String`
2. Update `input` format: `format!("{}:{}:{}", cpu, board, token_serial)`
3. No other changes needed

### Step 2 — payload.rs
1. Replace `LicensePayload` struct with:
```rust
#[derive(Debug, Deserialize)]
pub struct LicensePayload {
    pub expires_at: Option<i64>,
    pub issued_at: i64,
    pub issued_by: String,
    pub machine_fp: String,
    pub product: String,
}
```
2. Remove unused imports if any (e.g., `token_serial`/`version` were only Option fields — no import changes needed)

### Step 3 — mod.rs
1. Line 60: Change `machine::get_machine_fingerprint()` → `machine::get_machine_fingerprint(&token_serial)`
2. Lines 140-144: Replace `if let Some(ref licensed_fp) = license.machine_fp` block with direct compare:
```rust
if license.machine_fp != machine_fp {
    return Err(LicenseError::MachineMismatch);
}
```
3. Lines 147-151: **Delete** the entire `token_serial` check block (redundant — now embedded in fingerprint)
4. Lines 162-165: `LicenseInfo` construction — `license.product` is now `String` not `Option<String>`, wrap in `Some()`:
```rust
product: Some(license.product),
```
5. Verify `license.expires_at` and `license.issued_at` references still compile

### Step 4 — Build verification
```bash
cd F:/.PROJECT/.CAHTQT.CLIENT.PROJ/cahtqt-client/src-tauri && cargo build 2>&1
```

## Todo List
- [x] Update `get_machine_fingerprint` signature + hash input in machine.rs
- [x] Replace `LicensePayload` struct in payload.rs
- [x] Update `get_machine_fingerprint` call in mod.rs (pass `&token_serial`)
- [x] Update machine_fp binding check (direct String compare)
- [x] Remove token_serial check block from mod.rs
- [x] Fix `LicenseInfo` product field wrapping
- [x] Run `cargo build` — confirm clean compile

## Success Criteria
- `cargo build` completes with no errors
- `get_machine_fingerprint` takes `token_serial: &str` and includes it in hash
- `LicensePayload` matches server-side struct exactly
- No `token_serial` field in payload, no separate token_serial check in mod.rs

## Risk Assessment
- **Low risk** — pure Rust struct/function signature change, no external API surface
- Existing `license.dat` files will fail deserialization — expected/acceptable (v2 licenses required)

## Security Considerations
- Token serial now cryptographically bound into fingerprint — stronger binding than separate field comparison
- Hash truncation (first 8 bytes) unchanged — maintains same fingerprint length

## Next Steps
- Server must issue new licenses using v2 payload schema before deployment
- No other modules reference `LicensePayload` directly (verify with grep if needed)
