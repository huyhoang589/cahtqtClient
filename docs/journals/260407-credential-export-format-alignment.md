# Credential Export Format Alignment Complete

**Date**: 2026-04-07 14:54
**Severity**: Medium
**Component**: License system credential export
**Status**: Resolved

## What Happened

Aligned `export_machine_credential` output from desktop app to server-expected format. Task spanned two phases: Rust backend and TypeScript types.

## The Brutal Truth

Had to strip out client-side fields that felt useful but weren't in the spec. This is always uncomfortable — you build something, test it works, then learn the server doesn't care about it. But the discipline to follow contract specs saves integration pain later.

## Technical Details

**Rust changes** (`src-tauri/src/commands/license.rs`):
- Simplified `MachineCredentialResult` struct: removed `token_serial`, `user_name`, kept only `saved_path`
- Rewrote credential JSON to exact spec: `{token_serial, cpu_id, board_serial, registered_at}`
- Removed 7 old fields; added raw hardware IDs via `machine::get_cpu_id()` / `get_board_serial()`
- Timestamp format: `%Y-%m-%dT%H:%M:%SZ` (no fractional seconds, per spec)
- Deleted 26-line unused `read_first_cert_cn` helper
- Fixed TOCTOU issue: dual `Utc::now()` calls → single capture
- Replaced `.unwrap()` on `to_string()` with proper `map_err` error propagation

**TypeScript** (`src/types/index.ts`):
- Mirrored Rust struct changes: `MachineCredentialResult` now minimal

## What We Tried

Code review flagged three issues; all addressed:
- **M1 (TOCTOU)**: Two timestamp captures in same block → fixed with single `let now = Utc::now()`
- **M2 (Unsafe unwrap)**: Serialization `.unwrap()` in error path → replaced with proper `?` operator
- **H1 (Empty hardware IDs)**: Missing CPU/board IDs default to empty strings → accepted per spec validation (server validates these)

## Root Cause Analysis

Initial implementation leaked frontend concerns into the export format. Schema drift happens when there's no enforcement layer between client assumptions and server contracts. The fix: strict adherence to server-provided spec, empty strings for missing fields (server's validation problem, not ours).

## Lessons Learned

Empty field defaults beat optional fields when contracts are strict. The server expects specific JSON structure; `unwrap_or_default()` handles missing hardware IDs gracefully. Tests and code review catch integration mismatches before shipping.

## Next Steps

- Commit both phases to feature/license branch
- Ready for integration test with credential export/import flow
