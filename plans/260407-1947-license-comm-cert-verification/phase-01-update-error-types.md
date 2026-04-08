# Phase 1: Update Error Types

## Context Links
- [error.rs](../../src-tauri/src/license/error.rs) — LicenseError enum, LicenseStatus enum
- [plan.md](./plan.md)

## Overview
- **Priority:** P1 (blocks Phase 2+3)
- **Status:** Complete
- **Description:** Add `NoCommunicationCert` variant to both `LicenseError` and `LicenseStatus` enums so missing comm cert is a distinct, user-friendly error state.

## Key Insights
- Frontend already switches on `LicenseStatus` variants for UI display
- New variant needed in both enum + Display impl + `to_status()` mapping
- Frontend will need to handle new `no_communication_cert` status string (serde rename_all = snake_case)

## Requirements
### Functional
- `LicenseError::NoCommunicationCert` variant with descriptive message
- `LicenseStatus::NoCommunicationCert` variant for frontend display
- Display impl: "Communication certificate not configured. Please import the server certificate in Settings."
- `to_status()` maps `NoCommunicationCert` -> `LicenseStatus::NoCommunicationCert`

### Non-functional
- No breaking changes to existing variants
- Backwards compatible serde output

## Related Code Files
- **Modify:** `src-tauri/src/license/error.rs`

## Implementation Steps

1. Open `src-tauri/src/license/error.rs`
2. Add `NoCommunicationCert` to `LicenseStatus` enum (line ~14, after `Corrupted`):
   ```rust
   NoCommunicationCert,
   ```
3. Add `NoCommunicationCert` to `LicenseError` enum (line ~46, after `Corrupted(String)`):
   ```rust
   NoCommunicationCert,
   ```
4. Add arm to `Display` impl (line ~57 area):
   ```rust
   Self::NoCommunicationCert => write!(f, "Communication certificate not configured. Please import the server certificate in Settings."),
   ```
5. Add arm to `to_status()` (line ~73 area):
   ```rust
   Self::NoCommunicationCert => LicenseStatus::NoCommunicationCert,
   ```

## Todo List
- [x] Add `NoCommunicationCert` to `LicenseStatus`
- [x] Add `NoCommunicationCert` to `LicenseError`
- [x] Add Display arm
- [x] Add `to_status()` arm

## Success Criteria
- `cargo check` passes with new variants
- No warnings about non-exhaustive matches

## Risk Assessment
- **Low risk:** Additive change only, no existing match arms affected
- Frontend must handle new variant — but unknown status already falls through gracefully

## Security Considerations
- None — error type change only

## Next Steps
- Phase 2 depends on this (uses `LicenseError::NoCommunicationCert`)
