# Documentation Update Summary - CryptoModule Change v1

**Date:** 2026-03-06
**Version Updated:** 1.0.0-crypto-v1

## Overview

Updated all core documentation files to reflect the completed CryptoModule Change v1 implementation, which replaces the legacy `crypto_dll.dll` FFI bridge with new `htqt_crypto.dll` and implements a 3-state token PIN management system.

## Files Updated

### 1. `docs/system-architecture.md` (558 lines)
**Status:** Complete ✓

**Changes Made:**
- Updated version and last updated date (2026-03-06)
- Replaced architecture diagram to show `htqt_crypto.dll` instead of `crypto_dll.dll`
- Updated AppState structure with new `token_login: Arc<Mutex<TokenLoginState>>` field
- Added TokenLoginState enum documentation (Disconnected, Connected, LoggedIn)
- Documented new token management flow (login, cached PIN, logout)
- Replaced FFI Bridge section entirely:
  - New DLL contract for `encHTQT`, `decHTQT`, `HTQT_GetError`
  - Multi-certificate support per single call
  - PIN no longer passed per-operation
  - New output path structure with date stamps
- Updated command execution flow to show login modal → cached PIN → single DLL call
- Updated database schema to use `partners` and `partner_members` (reflects prior migration)
- Updated settings keys (removed DLL_PATH, kept pkcs11_library_path, sender_cn, sender_org)
- Updated frontend component tree to show TokenWarningBar and LoginTokenModal
- Updated security section for PIN caching vs. immediate zeroization
- Clarified that DLL path is auto-located (not user-configurable)

**Line Count:** 558 (was 506, +52 lines for new sections)

---

### 2. `docs/codebase-summary.md` (348 lines)
**Status:** Complete ✓

**Changes Made:**
- Updated version to 1.0.0-crypto-v1 and last updated date
- Updated core modules list: `htqt_ffi.rs` replaces `dll_wrapper.rs` and `dll_error.rs`
- Updated models.rs description to include TokenLoginState
- Updated eToken commands to include `login_token`, `logout_token`, `get_token_status`
- Updated encrypt/decrypt commands to show single-call pattern and output paths
- Added `token_status_changed` event documentation
- Updated frontend components: renamed `pin-dialog.tsx` to `login-token-modal.tsx`, added `token-warning-bar.tsx`
- Removed `dll-path-config.tsx` from components list
- Updated SettingsPage description (no DLL config)
- Updated Data Flow section entirely:
  - Encryption: login → single encHTQT call → .sf output
  - Decryption: similar flow with decHTQT → no-extension output
  - Added Logout flow
- Updated Security Highlights to emphasize PIN caching vs. per-operation zeroization
- Updated database settings keys (removed DLL_PATH)
- Updated file structure tree with htqt_ffi.rs and new command structure
- Updated component comments to show NEW features

**Line Count:** 348 (was 338, +10 lines for new sections)

---

### 3. `docs/project-changelog.md` (517 lines)
**Status:** Complete ✓

**Changes Made:**
- Added new version entry: `[1.0.0-crypto-v1] - 2026-03-06` at top
- Comprehensive changelog section covering:
  - **Added:** New htqt_ffi.rs, TokenLoginState, login/logout commands, LoginTokenModal, TokenWarningBar, new events
  - **Changed:** Single DLL call pattern, output filenames with .sf extension, token authentication as global session
  - **Removed:** Old dll_wrapper.rs, dll_error.rs, dll-path-config.tsx, PIN dialog from DecryptPage
  - **Breaking Changes:** DLL contract change, PIN flow change, output paths, settings changes
  - **Migration Path:** New DLL location, one-time login, output paths auto-created
  - **Security:** PIN not persisted, zeroized on logout/disconnect
  - **Performance:** Single DLL call per batch, token polling interval
  - **Testing:** Login/logout verified, status updates, output paths verified
- Updated Version History table to include 1.0.0-crypto-v1 at top
- Updated Statistics section (incremented Rust modules to 12, LOC, commands to 20, components to 25)

**Line Count:** 517 (was 428, +89 lines for new changelog entry)

---

## Key Documentation Themes

### 1. FFI Bridge Modernization
- Legacy `crypto_dll.dll` with per-operation PIN → **New `htqt_crypto.dll` with session PIN caching**
- Per-file DLL calls (M×N) → **Single multi-certificate call**
- Old API: EncryptFiles, DecryptFiles → **New API: encHTQT, decHTQT, HTQT_GetError**

### 2. Token PIN Management
- PIN no longer passed per encryption/decryption operation
- Introduced `TokenLoginState` enum (Disconnected → Connected → LoggedIn)
- PIN cached in `AppState.token_login` during session
- Zeroized on `logout_token()` command or automatic disconnect

### 3. Frontend UX Improvements
- **LoginTokenModal:** Orange Radix dialog appears once per session if token needed
- **TokenWarningBar:** Shows connection status on encrypt/decrypt pages
- **useTokenStatus Hook:** 20-second polling interval for real-time status
- Settings page simplified (no manual DLL path config)

### 4. Output Path Changes
- Encrypt: `DATA/ENCRYPT/{partner_name}/{stem}_{DDMMYYYY}.sf`
- Decrypt: `DATA/DECRYPT/{partner_name}/{stem}` (no extension)
- Paths auto-created on first use (no manual directory setup)

### 5. Database Schema (No Changes)
- Existing migration 003 covers prior renames
- Settings keys updated: DLL_PATH removed, sender_cn/sender_org retained
- No new migrations needed (DLL auto-located from exe directory)

---

## Verification Checklist

- [x] system-architecture.md updated with new FFI bridge details
- [x] system-architecture.md includes TokenLoginState and token flow
- [x] codebase-summary.md reflects htqt_ffi.rs and new commands
- [x] codebase-summary.md shows single-call DLL pattern
- [x] project-changelog.md has comprehensive v1.0.0-crypto-v1 entry
- [x] All file links verified (no broken references)
- [x] All component names accurate (LoginTokenModal, TokenWarningBar, etc.)
- [x] All command names accurate (login_token, logout_token, get_token_status)
- [x] Output paths correctly documented (DDMMYYYY format, .sf extension)
- [x] Version history updated
- [x] Statistics updated
- [x] No documentation exceeds 800 LOC limit

---

## Impact Summary

**Total Updates:** 3 core documentation files
**Total Lines Added:** ~151 lines across all files
**Files Size Status:** All files within 800 LOC limit
- system-architecture.md: 558 lines (68% utilization)
- codebase-summary.md: 348 lines (44% utilization)
- project-changelog.md: 517 lines (65% utilization)

**Documentation Coverage:** Comprehensive coverage of:
1. Architecture changes (FFI, AppState, token flow)
2. API changes (new commands, events, endpoints)
3. UI changes (new modals, warning bars, components)
4. Data flow (login → cache PIN → single DLL call → logout)
5. Breaking changes and migration path
6. Security implications
7. Performance characteristics

---

## Notes for Developers

1. **DLL Deployment:** `htqt_crypto.dll` must be in the same directory as the executable
2. **PIN Management:** No PIN persisted to disk; session-only in memory
3. **Token Polling:** 20-second interval; can be tuned in useTokenStatus hook
4. **Output Paths:** Auto-created; developers no longer need manual directory setup
5. **Error Handling:** Use HTQT_GetError() after encHTQT/decHTQT calls
6. **Backward Compatibility:** Old crypto_dll.dll NOT supported; full migration required

---

**Documentation Update Complete**
Status: Ready for deployment
Verified: All links, file names, command names, API signatures accurate
