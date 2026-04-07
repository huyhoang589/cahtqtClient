---
phase: 1
title: "Update Rust Command + Types"
status: done
priority: P1
effort: 40min
---

# Phase 1: Update Rust Command + Types

## Context

- Brainstorm: `plans/reports/brainstorm-260407-1354-credential-export-format-alignment.md`
- Spec: `feature/1. license/ClientMachineCredentialFormat.txt`

## Related Code Files

### Modify
- `src-tauri/src/commands/license.rs` — Main changes

### No Changes Needed
- `src-tauri/src/license/machine.rs` — Already exposes `get_cpu_id()` and `get_board_serial()` as pub

## Implementation Steps

### 1. Update `MachineCredentialResult` struct (line 24-28)

```rust
// BEFORE
#[derive(Debug, Serialize)]
pub struct MachineCredentialResult {
    pub saved_path: String,
    pub token_serial: String,
    pub user_name: String,
}

// AFTER
#[derive(Debug, Serialize)]
pub struct MachineCredentialResult {
    pub saved_path: String,
}
```

### 2. Rewrite `export_machine_credential` body (line 84-162)

Replace the credential JSON construction:

```rust
// REMOVE: machine fingerprint collection
// let machine_fp = machine::get_machine_fingerprint();

// REMOVE: user_name collection (lines 120-126 — session opening, CN reading)

// ADD: raw hardware IDs
let cpu_id = machine::get_cpu_id().unwrap_or_default();
let board_serial = machine::get_board_serial().unwrap_or_default();

// NEW credential JSON — exact server spec
let credential = serde_json::json!({
    "token_serial": token_serial,
    "cpu_id": cpu_id,
    "board_serial": board_serial,
    "registered_at": chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
});
```

**Remove from function:**
- `machine::get_machine_fingerprint()` call (line 89)
- Session opening for CN reading (lines 120-126)
- `user_name` from result (line 159)
- `"machine_fingerprint"`, `"user_name"`, `"exported_at"`, `"app_version"` from JSON

**Keep in function:**
- PKCS#11 path resolution (lines 92-106)
- Token serial retrieval (lines 113-117)
- Output directory resolution (lines 129-137)
- File writing logic (lines 149-153)
- App log emission (line 155)

### 3. Remove `read_first_cert_cn` helper (lines 231-256)

Delete entirely — no longer used after `user_name` removal.

### 4. Update return value

```rust
Ok(MachineCredentialResult {
    saved_path: save_path,
})
```

## Todo List

- [ ] Update `MachineCredentialResult` struct — remove `token_serial`, `user_name`
- [ ] Rewrite credential JSON to spec format: `token_serial`, `cpu_id`, `board_serial`, `registered_at`
- [ ] Remove `user_name` collection logic (session open, CN read)
- [ ] Remove `read_first_cert_cn` helper function
- [ ] Update return value
- [ ] Run `cargo check` to verify compilation

## Success Criteria

- `export_machine_credential` outputs exact spec JSON: `{token_serial, cpu_id, board_serial, registered_at}`
- No compilation errors
- License verification pipeline unaffected (still uses hash internally)

## Risk Assessment

- **Low**: Output-only change, no verification logic affected
- **Edge case**: If `cpu_id` or `board_serial` returns `None`, exports empty string — server must handle this
<!-- Updated: Validation Session 1 - timestamp format changed from to_rfc3339() to explicit format("%Y-%m-%dT%H:%M:%SZ") per spec -->
