# Phase 1: Comm Key Service Module

## Context
- [Brainstorm Report](../reports/brainstorm-260409-2022-comm-key-sf1-upgrade.md)
- [lib.rs](../../src-tauri/src/lib.rs) — startup init
- [lib_loader.rs](../../src-tauri/src/htqt_ffi/lib_loader.rs) — decrypt_one_sfv1 API
- [decrypt.rs](../../src-tauri/src/commands/decrypt.rs) — existing decrypt pattern

## Overview
- **Priority:** Critical — foundation for all other phases
- **Status:** Complete
- **Description:** Create centralized `comm_key_service` module with shared decrypt/cleanup logic used by SET KEY, LICENSE, and ENCRYPT flows.

## Key Insights
- `decrypt_one_sfv1()` requires: PKCS#11 session (lib, slot, PIN), own_cert_der, CryptoCallbacksV2
- DLL is NOT thread-safe — uses global `DLL_LOCK` mutex
- Temp cert output goes to a directory (DLL decides filename from .sf1 content)
- Existing decrypt pattern in `decrypt.rs` uses `open_token_session()` + `spawn_blocking`

## Architecture

### Service Functions
```rust
// comm_key_service.rs

/// Decrypt .sf1 communication key → returns temp cert path
/// Requires active PKCS#11 session (token must be logged in)
pub fn decrypt_comm_key(
    sf1_path: &str,
    temp_dir: &str,
    htqt_lib: &HtqtLib,
    pkcs11_lib: &str,
    slot_id: u64,
    pin: &str,
    own_cert_der: &[u8],
    app: AppHandle,
) -> Result<String, String>

/// Delete temp cert file (best-effort, log on error)
pub fn cleanup_temp_cert(cert_path: &str)

/// Startup: delete any leftover decrypted certs in temp dir
pub fn cleanup_orphaned_certs(temp_dir: &Path)

/// Get path to stored .sf1 file in COMM_KEY dir (if exists)
pub fn get_stored_comm_key_path(comm_key_dir: &Path) -> Option<PathBuf>
```

### Data Flow
```
.sf1 file → decrypt_one_sfv1() via DLL → temp cert in DATA/Certs/partners/
  → caller uses cert → cleanup_temp_cert() deletes it
```

## Related Code Files
- **Create:** `src-tauri/src/comm_key_service.rs`
- **Modify:** `src-tauri/src/lib.rs` (add `pub mod comm_key_service;`)

## Implementation Steps

1. Create `src-tauri/src/comm_key_service.rs` with:
   - `decrypt_comm_key()` — opens PKCS#11 session, calls `decrypt_one_sfv1`, returns output path
   - `cleanup_temp_cert()` — `std::fs::remove_file()` with error logging
   - `cleanup_orphaned_certs()` — iterate temp dir, delete `.crt`/`.cer`/`.pem` files
   - `get_stored_comm_key_path()` — glob `DATA/COMM_KEY/*.sf1`, return first match

2. In `decrypt_comm_key()`:
   - Use same pattern as `run_decrypt_batch()` in decrypt.rs
   - `open_token_session()` for PKCS#11 context
   - Build `CryptoCallbacksV2` with `rsa_dec_fn = Some(cb_rsa_oaep_decrypt)`
   - Call `lib.decrypt_one_sfv1(sf1_path, temp_dir, &cbs, 0)`
   - Return the output path string on success
   - This runs in `spawn_blocking` context (caller handles that)

3. Add `pub mod comm_key_service;` to `lib.rs`

## Todo
- [x] Create `comm_key_service.rs` with all 4 functions
- [x] Add module declaration in `lib.rs`
- [x] Compile check: `cargo check`

## Success Criteria
- Module compiles without errors
- Functions have correct signatures matching existing decrypt patterns
- No duplicated logic from decrypt.rs (shared via this service)

## Risk Assessment
- `decrypt_one_sfv1` output filename is DLL-determined — must handle unknown filename in temp dir
- DLL_LOCK contention if decrypt called during another operation — mitigated by `is_operation_running` atomic

## Security Considerations
- Temp cert contains private key material — cleanup MUST happen even on error
- PIN passed as `&str` reference only, not stored in service
