# Code Review: Credential Export Format Alignment

**Date:** 2026-04-07
**Reviewer:** code-reviewer
**Scope:** `src-tauri/src/commands/license.rs`, `src/types/index.ts`
**LOC changed:** ~30 (Rust), ~3 (TypeScript)

---

## Overall Assessment

Clean, focused change. Struct simplified correctly, old helper deleted, consumers only use `saved_path` -- no dangling references. Two issues found: one high-priority (silent hardware ID failure), one medium (TOCTOU on timestamp).

---

## Critical Issues

None.

---

## High Priority

### H1. Both `cpu_id` and `board_serial` can silently be empty strings (license.rs:87-88)

`get_cpu_id()` and `get_board_serial()` return `Option<String>`. The code uses `unwrap_or_default()`, producing empty strings when hardware IDs are unavailable (VM, restricted permissions, `wmic` absent). The credential JSON would contain `"cpu_id": ""` which the server may reject or silently bind to an invalid machine.

**Impact:** Credential file looks valid but contains empty identifiers -- server may issue a license that cannot be verified later, or reject silently.

**Recommendation:** Fail early if either hardware ID is missing:

```rust
let cpu_id = machine::get_cpu_id()
    .ok_or("Cannot read CPU ID. Run as Administrator or check wmic availability.")?;
let board_serial = machine::get_board_serial()
    .ok_or("Cannot read motherboard serial. Run as Administrator or check wmic availability.")?;
```

If empty-string fallback is intentional for VMs, document that decision and validate on the server side.

---

## Medium Priority

### M1. Two separate `Utc::now()` calls produce different timestamps (license.rs:129-130)

```rust
let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
let registered_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
```

These are two distinct clock reads. In the rare case the call spans a second boundary, the filename timestamp and `registered_at` field will disagree (e.g., file says `20260407_235959` but JSON says `2026-04-08T00:00:00Z`).

**Recommendation:** Capture once:

```rust
let now = chrono::Utc::now();
let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
let registered_at = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
```

### M2. `serde_json::to_string_pretty(&credential).unwrap()` can panic (license.rs:141)

The `serde_json::json!` macro produces a `Value` that should always serialize, but using `.unwrap()` in a Tauri command means any serialization edge case (unlikely but possible with non-UTF8 data from `wmic`) would crash the app instead of returning an error.

**Recommendation:**

```rust
let json_str = serde_json::to_string_pretty(&credential)
    .map_err(|e| format!("Failed to serialize credential: {}", e))?;
std::fs::write(&save_path, json_str)
    .map_err(|e| format!("Failed to write credential file: {}", e))?;
```

---

## Low Priority

### L1. Trailing comma in `serde_json::json!` macro (license.rs:136)

```rust
"registered_at": registered_at,  // trailing comma
```

Valid Rust, no functional issue. Just noting for style consistency.

---

## Security Review

### S1. Raw hardware IDs written to disk (by design)

Previous review flagged this. The new format exports `cpu_id` and `board_serial` in plaintext to the user's chosen output directory. This is the server spec requirement so it's accepted, but worth noting:

- These are stable machine identifiers; if the credential file is leaked, the machine is permanently fingerprintable.
- The file is saved with default permissions -- any local user can read it.
- **No regression from previous version** (it already wrote hardware data, just in hashed form).

### S2. No path traversal risk on `save_path`

`output_data_dir` comes from user settings DB, `filename` is hardcoded format with timestamp. No user-controlled path components. Acceptable.

---

## Scout Findings: Edge Cases

1. **`wmic` deprecation on Windows 11** -- `wmic` is deprecated and may be removed in future Windows versions. `get_cpu_id()` and `get_board_serial()` will return `None`, hitting the empty-string issue from H1.
2. **Frontend consumers safe** -- `LicenseSection.tsx:42` and `license-screens.tsx:53` only access `result.saved_path`. No references to removed `token_serial` or `user_name` fields anywhere in `src/`.
3. **`read_first_cert_cn` fully removed** -- no remaining references in `src-tauri/`.
4. **`get_token_serial` in `license/token.rs`** -- public function, correctly used. Returns `LicenseError` which is converted via `map_err` at call site.

---

## Positive Observations

- Clean struct simplification -- Rust and TypeScript types match exactly
- Proper error propagation with `map_err` throughout
- Path traversal protection already exists on `import_license_file` (line 160)
- Dead code (`read_first_cert_cn`) properly removed

---

## Recommended Actions

1. **(High)** Fail early when hardware IDs are empty instead of `unwrap_or_default()` -- or explicitly document the empty-string contract with the server
2. **(Medium)** Capture `Utc::now()` once for both timestamp uses
3. **(Medium)** Replace `.unwrap()` with `?` on `to_string_pretty`
4. **(Future)** Consider migrating from `wmic` to `Get-CimInstance` PowerShell or Win32 API for hardware ID collection before Windows removes `wmic`

---

## Unresolved Questions

1. Does the server accept empty strings for `cpu_id` / `board_serial`? If not, H1 is a blocking issue.
2. Is the `%Y-%m-%dT%H:%M:%SZ` format confirmed as exact server spec, or does the server expect milliseconds (`%Y-%m-%dT%H:%M:%S%.3fZ`)?
