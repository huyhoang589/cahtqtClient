# Phase 5: Startup Orphan Cleanup

## Context
- [Phase 1](./phase-01-comm-key-service.md) — cleanup_orphaned_certs function
- [lib.rs](../../src-tauri/src/lib.rs) — initialize_data_directories (lines 40-58)

## Overview
- **Priority:** Medium
- **Status:** Complete
- **Description:** Add orphan cert cleanup at startup + create DATA/COMM_KEY dir.

## Implementation Steps

1. In `initialize_data_directories()` in `lib.rs`:
   - Add `data.join("COMM_KEY")` to the dirs array
   - After dir creation, call `comm_key_service::cleanup_orphaned_certs(&data.join("Certs").join("partners"))`

2. `cleanup_orphaned_certs()` logic:
   - Iterate files in temp dir matching `*.crt`, `*.cer`, `*.pem`, `*.der`
   - Check if file is NOT referenced by `communication_cert_path` setting (don't delete the .sf1)
   - Actually: the .sf1 is in COMM_KEY dir, temp certs are in Certs/partners/ — safe to delete non-referenced files
   - Simple approach: delete files that were created/modified within last 24h AND are not referenced in DB
   - Simpler approach: just delete files in a dedicated temp subdir (but spec says use Certs/partners/)
   - **Simplest:** Delete all `.crt`/`.cer`/`.pem` files in Certs/partners/ that are NOT referenced by any PartnerMember's `cert_file_path` in DB
   - Wait — this requires DB access. At startup, DB is initialized before this cleanup.
   - **Decision:** Run cleanup AFTER DB init, not inside `initialize_data_directories()`

3. In `lib.rs` setup, after DB init + before AppState creation:
   ```rust
   // Cleanup orphaned temp certs from crash recovery
   comm_key_service::cleanup_orphaned_certs(
       &app_data_dir.join("DATA").join("Certs").join("partners"),
       &pool, // need DB to check referenced certs
   );
   ```

## Todo
- [x] Add COMM_KEY dir to initialize_data_directories
- [x] Add cleanup call after DB init in lib.rs setup
- [x] Compile check

## Success Criteria
- DATA/COMM_KEY/ created at startup
- Orphaned temp certs deleted at startup
- Referenced cert files (PartnerMember) NOT deleted

## Risk Assessment
- Low risk — cleanup is best-effort, errors logged not propagated
