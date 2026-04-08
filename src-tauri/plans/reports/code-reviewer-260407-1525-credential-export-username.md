# Code Review: export_machine_credential refactor

## Scope
- Files: `src-tauri/src/commands/license.rs` (primary), `etoken/certificate_reader.rs`, `etoken/token_manager.rs`, `license/token.rs` (context)
- LOC changed: ~60 (net reduction after removing `read_first_cert_cn`)
- Focus: correctness, security, edge cases, backwards compat

## Overall Assessment

Clean refactor. Consolidates slot enumeration, removes duplicated cert-reading logic, adds `user_name` + raw hardware IDs to credential JSON. Frontend TS types already updated to match. No critical issues found.

## Critical Issues

None.

## High Priority

### H1: Raw hardware IDs (cpu_id, board_serial) written to plaintext JSON on disk

**Before:** Only a truncated SHA-256 hash (`machine_fingerprint`) was exported.
**After:** Raw `cpu_id` (WMIC ProcessorId) and `board_serial` are written in cleartext to the credential JSON file.

**Impact:** These are stable hardware identifiers. If the credential file is leaked or shared beyond IT, it exposes machine-identifying data. This may be intentional per server spec, but worth confirming the server truly needs raw values vs the hash.

**Recommendation:** If server spec requires raw IDs, document this as an accepted risk. Consider whether the file should have restricted ACLs (`std::fs::set_permissions` or Windows DACL).

### H2: `slot_id: 0` hardcoded in `read_all_certificates(&session, 0)`

Line 129: `certificate_reader::read_all_certificates(&session, 0)` passes `0` as `slot_id`. Looking at `certificate_reader.rs`, this param is only used in error messages (`format!("slot {}")`), so functionally harmless. But it's misleading -- the actual slot could be any index. Should pass the real slot index for accurate diagnostics.

**Fix:**
```rust
// Extract the slot index from the slot_infos or pass the raw slot
let certs = certificate_reader::read_all_certificates(&session, 0) // slot_id only used in error msg
```
At minimum, pass `0` since we took `raw_slots.first()`, so index 0 is correct. This is actually fine as-is. Downgrading to informational.

## Medium Priority

### M1: No directory existence check before writing credential file

Line 160-164: `std::fs::write(&save_path, json_str)` will fail if `output_data_dir` doesn't exist. The error message will be "Failed to write credential file: ..." which is acceptable UX, but `std::fs::create_dir_all` would be more robust.

### M2: `unwrap_or_default()` for cpu_id/board_serial silently produces empty strings

Lines 88-89: If WMIC fails (e.g., on non-Windows or restricted environment), both fields become `""`. The server may reject empty values or treat them as valid -- unclear. The old code used `"UNAVAIL"` as fallback in `get_machine_fingerprint`. Consider matching that pattern or returning an error if hardware IDs are required by server spec.

### M3: Session lifetime and PKCS#11 context not explicitly finalized

The `Pkcs11` context created at line 112 is dropped at end of function. `cryptoki` should call `C_Finalize` on drop, but if multiple concurrent calls to `export_machine_credential` occur (user clicks export rapidly), each creates a separate `Pkcs11` context. The PKCS#11 spec says `C_Initialize`/`C_Finalize` should be called once per process. Practically unlikely to cause issues with Bit4ID middleware, but worth noting.

**Recommendation:** Consider caching the `Pkcs11` context in `AppState` if this becomes a recurring pattern.

## Low Priority

### L1: `registered_at` format duplicates `chrono::Utc::now().to_rfc3339()`

Line 150: Manual format `"%Y-%m-%dT%H:%M:%SZ"` produces RFC3339 without fractional seconds. Using `now.to_rfc3339_opts(SecondsFormat::Secs, true)` would be more explicit and self-documenting. Minor style preference; current code is correct.

### L2: Trailing comma in `serde_json::json!` macro

Line 157: `"registered_at": registered_at,` -- trailing comma is valid in Rust macros but inconsistent with JSON style. No functional impact.

## Edge Cases Verified

- **No token inserted:** Handled -- `raw_slots.first()` returns `None`, produces clear error message (line 118)
- **Token with no certs:** Handled -- `read_all_certificates` returns empty vec, `certs.first()` returns `None`, `user_name` becomes `""` (line 133)
- **Token with only CA certs:** Same as above -- `certificate_reader` filters out CA certs, result is empty
- **Frontend TS types:** Already updated at `src/types/index.ts:240` -- `MachineCredentialResult` has only `saved_path`. No breaking change.
- **`get_token_serial` in `license/token.rs`:** Still exists and used by other callers (`is_licensed` flow). No dead code introduced in this diff.

## Positive Observations

- Eliminated duplicated slot enumeration (was calling `get_slots_with_token()` twice)
- Removed inline `read_first_cert_cn` in favor of reusing `certificate_reader::read_all_certificates` -- DRY
- Proper error propagation with `map_err` throughout
- `serde_json::to_string_pretty` now has explicit error handling instead of `.unwrap()`
- Empty-string fallback for `user_name` is consistent with existing patterns

## Recommended Actions

1. **Confirm with server team:** raw cpu_id/board_serial is intentional (vs hash) -- document accepted risk
2. **Consider:** `std::fs::create_dir_all` before writing credential file (M1)
3. **Consider:** returning error instead of empty string when hardware IDs unavailable, if server requires them (M2)

## Unresolved Questions

1. Does the server validate or reject credentials with empty `cpu_id`/`board_serial` fields?
2. Is there a file sensitivity classification for the exported credential JSON? Should it have restricted file permissions?
