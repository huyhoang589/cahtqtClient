---
phase: 2
title: "Rust Tauri Commands"
status: done
effort: 1.5h
depends_on: [1]
completed: 2026-04-07
---

# Phase 2: Rust Tauri Commands

## Context Links
- [Plan](plan.md) | [Phase 1](phase-01-rust-license-backend.md)
- `src-tauri/src/commands/settings.rs` — existing command patterns
- `src-tauri/src/commands/etoken.rs` — PKCS#11 command patterns
- `src-tauri/src/lib.rs` — AppState struct + invoke_handler registration

## Overview
- **Priority**: P1 — bridges backend license logic to frontend
- **Status**: pending
- **Description**: Create Tauri commands for license info, credential export, license import, and startup check. Add LicenseState to AppState cache.

## Key Insights
- Follow existing pattern: `#[tauri::command]` async fns in `commands/` module
- AppState already has `Arc<Mutex<...>>` pattern for cached data (see `last_token_scan`)
- `output_data_dir` resolution already implemented in `commands/settings.rs` — reuse for credential export
- `check_license` runs at startup in `setup()` — result cached in AppState for `get_license_info`

## Requirements

### Functional
- `check_license` — full 11-step verification, returns LicenseCheckResult
- `get_license_info` — reads cached state, works even without license (returns status)
- `export_machine_credential` — collects hardware IDs + token info, saves JSON to output_data_dir
- `import_license_file` — validates + copies license.dat to AppData, refreshes cached state

### Non-Functional
- Commands return `Result<T, String>` (Tauri convention)
- No blocking the main thread — async where I/O involved

## Architecture

```
src-tauri/src/commands/license.rs    — 4 Tauri commands
src-tauri/src/lib.rs                 — AppState gains license_info field
```

## Related Code Files

### Files to Create
- `src-tauri/src/commands/license.rs`

### Files to Modify
- `src-tauri/src/commands/mod.rs` — add `pub mod license;`
- `src-tauri/src/lib.rs` — add `license_info: Arc<Mutex<LicenseInfo>>` to AppState, register commands, run startup check in `setup()`

## Implementation Steps

### 1. Add LicenseInfo to AppState (`lib.rs`)
```rust
pub struct AppState {
    // ... existing fields ...
    pub license_info: Arc<Mutex<license::error::LicenseInfo>>,
}
```
Initialize with `LicenseInfo { status: NotFound, expires_at: None, product: None }` then update during setup.

### 2. Run license check at startup (`lib.rs` setup closure)
After DB init, before managing AppState:
```rust
let license_info = license::is_licensed(&pkcs11_path, &app_data_dir);
```
Store result in AppState. This runs the full 11-step pipeline once.

### 3. Create `commands/license.rs` with 4 commands

#### `check_license` → LicenseCheckResult
<!-- Updated: Validation Session 1 - Shared PKCS#11 context + debug bypass -->
- Called by LicenseGate frontend component on mount
- Reads cached `license_info` from AppState
- Uses shared PKCS#11 context from AppState (not a separate session)
- `#[cfg(debug_assertions)]` bypass: return `ok` state immediately in debug builds
- Returns `{ state: "ok"|"no_token"|"no_license"|"error", error_msg: Option<String> }`
- Maps LicenseStatus enum to state strings

#### `get_license_info` → LicenseInfo
- Called by Settings LicenseSection on mount
- Reads cached `license_info` from AppState
- Returns `{ status, expires_at, product }`
- Works even when license missing (returns `not_found` status)

#### `export_machine_credential` → MachineCredentialResult
- Calls `license::machine::get_cpu_id()`, `get_board_serial()`
- Reads token serial via PKCS#11 `C_GetTokenInfo` (no PIN needed)
- Reads CN from token certificate (public object, no PIN needed)
- Resolves output_data_dir from settings (reuse `get_app_settings` logic)
- Writes JSON to `{output_data_dir}/machine_credential_{timestamp}.json`
- Returns `{ saved_path, token_serial, user_name }`
- Errors if token not inserted or hardware IDs unavailable

#### `import_license_file` → ImportLicenseResult
- Accepts `file_path: String` (selected via frontend file picker)
- Validates:
  - File exists and is non-empty
  - Base64-decodes successfully
  - Contains `||SIG||` separator
  - RSA signature verifies against embedded server public key
- Copies validated file to `{app_data_dir}/license.dat`
- Re-runs verification pipeline to update cached state
- Returns `{ status, expires_at }` with new license info
- Does NOT require token present (validates structure only, full check on next launch)

### 4. Register commands in `lib.rs` invoke_handler
```rust
commands::license::check_license,
commands::license::get_license_info,
commands::license::export_machine_credential,
commands::license::import_license_file,
```

### 5. Update `commands/mod.rs`
```rust
pub mod license;
```

## Todo List
- [x] Add `license_info` field to AppState in lib.rs
- [x] Add startup license check in setup() closure
- [x] Create commands/license.rs with 4 commands
- [x] Register commands in invoke_handler
- [x] Add `pub mod license;` to commands/mod.rs
- [x] Compile check passes

## Success Criteria
- `cargo check` passes
- All 4 commands registered and callable from frontend
- `get_license_info` returns valid response even without license.dat
- `export_machine_credential` writes correct JSON format
- `import_license_file` rejects invalid files, accepts valid ones

## Risk Assessment
- **Startup performance**: License check adds to app launch time. Mitigation: spec says < 300ms is acceptable; challenge-response is the slowest part. If token not present, early-exit is fast.
- **File overwrite on import**: Importing over existing license.dat. Mitigation: validate before overwriting; could backup old file but YAGNI for now.
- **PKCS#11 session conflict**: Export credential reads token while etoken module may have active session. Mitigation: use separate PKCS#11 session or reuse existing session from AppState.

## Security Considerations
- `import_license_file` validates RSA signature BEFORE persisting — prevents arbitrary file injection
- File path from frontend: use only filename component for any path operations (prevent path traversal)
- Credential export writes to user-selected output_data_dir — no escalation risk
- PIN not required for export (only public PKCS#11 operations)
