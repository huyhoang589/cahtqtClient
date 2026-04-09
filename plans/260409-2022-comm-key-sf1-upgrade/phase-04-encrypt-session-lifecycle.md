# Phase 4: Encrypt Per-Session Decrypt Lifecycle

## Context
- [Phase 1](./phase-01-comm-key-service.md) — comm_key_service dependency
- [encrypt.rs](../../src-tauri/src/commands/encrypt.rs) — current encrypt batch
- [EncryptPage.tsx](../../src/pages/EncryptPage.tsx) — current encrypt UI

## Overview
- **Priority:** High
- **Status:** Complete
- **Description:** Wrap encrypt batch with communication key decrypt/cleanup lifecycle. Each encrypt session: decrypt .sf1 → temp cert → encrypt files with cert → delete temp cert.

## Key Insights
- Current flow: `encrypt_batch()` receives `cert_paths` from frontend (pre-resolved cert file paths)
- New flow: backend decrypts .sf1 to get cert path, passes to DLL, then cleans up
- Encrypt already runs in `spawn_blocking` — decrypt can happen before the DLL call
- `is_operation_running` atomic prevents concurrent operations (protects DLL_LOCK)
- Cleanup MUST happen even on DLL error — use Rust's drop guard or explicit finally

## Architecture

### Modified Encrypt Flow
```
encrypt_batch() called:
  1. Read comm key .sf1 path from settings DB
  2. decrypt_comm_key(.sf1, temp_dir) → temp_cert_path (in spawn_blocking)
  3. Use temp_cert_path as the single recipient cert
  4. encHTQT_sf_multi(files, [temp_cert_path]) — existing DLL call
  5. cleanup_temp_cert(temp_cert_path) — ALWAYS, even on error
  6. Return result
```

### Frontend Changes
- EncryptPage no longer passes `cert_paths` / `commCert.file_path` to backend
- Backend reads comm key path from DB internally
- Frontend only needs to know if comm key is valid (status check)
- `startEncrypt()` signature simplifies: no cert_paths parameter

### Cleanup Guard
```rust
struct TempCertGuard { path: Option<String> }
impl Drop for TempCertGuard {
    fn drop(&mut self) {
        if let Some(ref p) = self.path {
            comm_key_service::cleanup_temp_cert(p);
        }
    }
}
```

## Related Code Files
- **Modify:** `src-tauri/src/commands/encrypt.rs` — add .sf1 decrypt/cleanup in `run_encrypt_batch()`
- **Modify:** `src/pages/EncryptPage.tsx` — remove cert_paths from encrypt call
- **Modify:** `src/hooks/use-encrypt.ts` — update startEncrypt signature
- **Modify:** `src/lib/tauri-api.ts` — update encryptBatch API call

## Implementation Steps

### Backend
1. In `run_encrypt_batch()`, before building CString arrays:
   - Read `communication_cert_path` from settings (the .sf1 path in COMM_KEY dir)
   - If missing/empty → return error "Communication key not set"
   - In the `spawn_blocking` block, before DLL call:
     - Decrypt .sf1: `comm_key_service::decrypt_comm_key(...)` → temp_cert_path
     - Create `TempCertGuard` for cleanup on drop
     - Use temp_cert_path as the single cert in `cert_paths`

2. Update `encrypt_batch` signature:
   - Remove `cert_paths` parameter (backend resolves internally)
   - Keep `partner_name` (can derive from saved recipient info)
   - Or simplify to just `src_paths` + optional `output_dir`

3. Add `TempCertGuard` struct for RAII cleanup

### Frontend
4. Update `EncryptPage.tsx`:
   - Remove `commCert.file_path` from `startEncrypt()` call
   - Keep `commCert` state for display (recipient banner)
   - `canEncrypt` still checks `commCert !== null` (UI guard)

5. Update `use-encrypt.ts` hook:
   - `startEncrypt()` no longer takes cert_paths
   - Calls `encryptBatch(srcPaths)` (simplified)

6. Update `tauri-api.ts`:
   - `encryptBatch()` no longer passes cert_paths

## Todo
- [x] Backend: add .sf1 decrypt in `run_encrypt_batch()`
- [x] Backend: add TempCertGuard for cleanup
- [x] Backend: update encrypt_batch signature (remove cert_paths)
- [x] Frontend: update EncryptPage.tsx
- [x] Frontend: update use-encrypt.ts
- [x] Frontend: update tauri-api.ts
- [x] Compile check

## Success Criteria
- Encrypt batch decrypts .sf1 before DLL call
- Temp cert cleaned up after ALL files processed (success or error)
- Temp cert cleaned up even on DLL panic (via Drop guard)
- Frontend no longer passes cert paths to backend
- Encrypt still works end-to-end with same output files

## Risk Assessment
- Token session already open for encrypt signing → can reuse for .sf1 decrypt? No — separate session. But DLL_LOCK prevents concurrent. Decrypt happens first, then encrypt session opens.
- Actually: decrypt needs its own PKCS#11 session, then encrypt opens another. Both within spawn_blocking, sequential.

## Security Considerations
- TempCertGuard ensures cert deletion even on panic
- Cert only exists in memory/disk during encrypt operation
