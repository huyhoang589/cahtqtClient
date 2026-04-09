# Phase 3: License Startup with .sf1 Decrypt

## Context
- [Phase 1](./phase-01-comm-key-service.md) — comm_key_service dependency
- [license/mod.rs](../../src-tauri/src/license/mod.rs) — current license verification
- [lib.rs](../../src-tauri/src/lib.rs) — startup license check (lines 114-135)

## Overview
- **Priority:** High
- **Status:** Complete
- **Description:** Modify license startup verification to decrypt .sf1 communication key first, use the temp cert for signature verification, then delete temp cert.

## Key Insights
- Current flow: read `communication_cert_path` → read cert file → extract pubkey → verify signature
- New flow: read `communication_cert_path` (now points to .sf1) → decrypt .sf1 → temp cert → extract pubkey → verify → delete temp cert
- Startup runs in `tauri::async_runtime::block_on` — synchronous context
- PKCS#11 token needed at startup for .sf1 decryption → user must enter PIN
- Current `is_licensed()` is a pure sync function — needs access to DLL for decrypt

## Architecture

### Modified License Flow
```
Startup:
  1. Read communication_cert_path from DB (now .sf1 path)
  2. Check .sf1 exists → if not, return NoCommunicationCert
  3. Initialize PKCS#11 (same as current Step 1-2)
  4. decrypt_comm_key(.sf1_path, temp_dir) → temp cert path
  5. Read temp cert → extract RSA pubkey (same as current Step 6)
  6. Verify license.dat signature (same as current Step 7+)
  7. cleanup_temp_cert(temp_cert_path)
  8. Return LicenseInfo
```

### Challenge: Sync vs Async
- `is_licensed()` is sync, called from `block_on` at startup
- `decrypt_comm_key()` needs DLL access (also sync via DLL_LOCK)
- Solution: keep license verification sync — `decrypt_one_sfv1` is already sync
- The comm_key_service `decrypt_comm_key` can have a sync variant for startup use

### PIN at Startup
- At startup, no token login state exists yet
- Options:
  a) Auto-detect PKCS#11 lib + open session with no PIN (some tokens allow RO without login)
  b) Defer license check to after first token login
  c) Store encrypted PIN (not recommended)
- **Chosen: Option B** — defer license verification until after token login, re-check when login_token completes
- Startup: set license status to "pending" if comm key exists but no token session
- After `login_token` command succeeds → trigger license re-validation with now-available PIN

## Related Code Files
- **Modify:** `src-tauri/src/license/mod.rs` — add .sf1 decrypt step in `verify_full()`
- **Modify:** `src-tauri/src/lib.rs` — update startup license flow
- **Modify:** `src-tauri/src/commands/license.rs` — add re-validation command or hook into login
- **Modify:** `src-tauri/src/license/error.rs` — add `Pending` status if needed

## Implementation Steps

1. Add `LicenseStatus::Pending` variant to `error.rs` (license not yet checked, waiting for token login)

2. Modify `is_licensed()` signature to accept optional HtqtLib + token session params:
   ```rust
   pub fn is_licensed(
       pkcs11_lib_path: &str,
       app_data_dir: &Path,
       comm_key_sf1_path: Option<&str>,
       htqt_lib: Option<&HtqtLib>,
       token_session: Option<(u64, &str, &[u8])>, // (slot, pin, own_cert_der)
   ) -> LicenseInfo
   ```

<!-- Updated: Validation Session 1 - Clean migration, .sf1 only, no backward compat -->
3. In `verify_full()`, before Step 6 (extract pubkey):
   - `comm_key_sf1_path` is always .sf1 (no backward compat for plain certs)
   - Need HtqtLib + token session → if not available, return `LicenseStatus::Pending`
   - Call `comm_key_service::decrypt_comm_key()` (sync variant)
   - Read decrypted temp cert
   - Extract pubkey from temp cert
   - Cleanup temp cert

4. At startup in `lib.rs`:
   - If comm_cert_path points to .sf1 and no token session → set `Pending`
   - License will be re-validated when user logs into token

5. Add `revalidate_license` Tauri command (or hook into existing `login_token`):
   - Called after successful token login
   - Has access to PKCS#11 session + PIN + own_cert_der
   - Calls `is_licensed()` with full params
   - Updates `state.license_info`
   - Emits `license-changed` event

6. Frontend: handle `Pending` status — show "License check pending — login to token" message

## Todo
- [x] Add `LicenseStatus::Pending` variant
- [x] Modify `is_licensed()` / `verify_full()` to handle .sf1 decrypt
- [x] Update startup flow in `lib.rs`
- [x] Add license re-validation after token login
- [x] Frontend: handle Pending status display
- [x] Compile check

## Success Criteria
- Startup: if .sf1 comm key exists but no token → status = Pending
- After token login: license re-validates using decrypted .sf1 cert
- License.dat signature verified against cert extracted from .sf1
- Temp cert cleaned up after verification
- Clean migration: only .sf1 paths supported (no backward compat for plain certs)

## Risk Assessment
- User never logs into token → license stays Pending → encrypt/decrypt blocked by per-route guards (acceptable)
- Token login fails → license stays Pending → user sees clear message
- .sf1 decrypt fails → license status = Invalid with error message

## Security Considerations
- Temp cert exists only during verification (milliseconds)
- PIN accessed from token_login state (Zeroizing)
- No cert material cached in memory after cleanup
