# Project Changelog

All notable changes to CAHTQT Client are documented here, following semantic versioning.

---

## [Unreleased] — feature/license Branch

### Added

- **License Signature Verification Refactor**
  - `extract_public_key_from_cert()` function for runtime RSA public key extraction from X.509 certificates
  - Support for both PEM (Base64-encoded) and DER (binary) certificate formats with auto-detection
  - `NoCommunicationCert` error variant for missing or unconfigured communication certificates
  - `NoCommunicationCert` status variant in `LicenseStatus` enum for frontend display

### Changed

- **License Module API (Breaking Change for Internal Code)**
  - `verify_license_signature()` now takes `&RsaPublicKey` parameter instead of reading from hardcoded constant
  - `is_licensed()` now accepts `comm_cert_path: Option<&str>` parameter (read from SQLite settings)
  - Removed `#[cfg(debug_assertions)]` guard that was skipping signature verification in debug builds
  - All builds now enforce RSA signature verification (dev and release)

- **Command: import_license_file**
  - Now re-runs full license verification after importing, using active communication certificate path
  - Properly returns `NoCommunicationCert` error if certificate path not configured

- **Certificate Handling**
  - Path traversal validation: Reject relative paths and `..` directory traversal attempts
  - File existence check before reading certificate
  - X.509 parsing validates certificate structure before key extraction
  - RSA key type validation (rejects non-RSA certificates)

### Security Improvements

- Eliminated hardcoded public key constant from source code (was guarded by `compile_error!`)
- Public key now derived from configurable server certificate at runtime
- Supports certificate rotation without code recompilation
- Path safety validation on all certificate file operations

### Documentation

- System architecture document created with full license verification pipeline
- Code standards document with module organization and naming conventions
- Project overview and PDR documenting functional/non-functional requirements
- This changelog for tracking project evolution

---

## [1.0.0] — 2024-04-07 (Implicit Release from Commits)

### Added (From Earlier Commits)

**Two-Factor Hardware-Bound License System (2F-HBLS)** — Commit 1e1f7f2
- Token verification via PKCS#11 challenge-response
- License file signature verification with RSA-PKCS1v15-SHA256
- Machine fingerprint binding (SHA256 of CPU ID + Board Serial + MAC addresses)
- Token serial binding for hardware lock
- License expiry validation with Unix timestamp
- Caching of license state in AppState for frontend consumption
- Debug-build bypass (auto-pass license check in debug assertions)

**Credential Export Format Alignment** — Commit af5bf9a
- Machine credential export matching server-expected JSON spec
- Collects: board_serial, cpu_id, token_serial, registered_at
- Saves to user-configurable output directory
- Timestamp-based file naming (machine_credential_YYYYMMDD_HHMMSS.json)

**User Identity in Credential Export** — Commit 464c510
- Extract user_name from first non-CA certificate's CN (Common Name)
- Added to machine credential JSON payload
- Read from token via certificate_reader module
- Enables server-side user tracking in license binding

---

## Technical Debt & Known Issues

### Resolved in This Refactor

- **Hardcoded Secret:** Removed `SERVER_PUBLIC_KEY_PEM` placeholder with `compile_error!` guard
- **Inflexible Key Management:** Moved from compile-time constant to runtime certificate loading
- **Debug-Build Bypass:** Removed license verification bypass in debug builds

### Remaining Items (Backlog)

- No certificate revocation checking (CRL/OCSP)
- No system time validation against certificate validity
- Single token support (multi-token scenarios untested)
- Challenge-response may fail if token requires PIN (continues without it)
- No hardware security module (HSM) integration

---

## Breaking Changes

### For Internal Code

**Function Signature Changes:**
```rust
// Before
pub fn verify_license_signature(payload: &[u8], sig: &[u8]) -> Result<(), LicenseError>

// After
pub fn verify_license_signature(
    payload: &[u8],
    sig: &[u8],
    public_key: &RsaPublicKey,
) -> Result<(), LicenseError>
```

**Parameter Addition:**
```rust
// Before
pub fn is_licensed(pkcs11_lib_path: &str, app_data_dir: &Path) -> LicenseInfo

// After
pub fn is_licensed(
    pkcs11_lib_path: &str,
    app_data_dir: &Path,
    comm_cert_path: Option<&str>,
) -> LicenseInfo
```

### For Deployment

- Requires communication certificate path configuration in SQLite settings
- No hardcoded fallback — missing cert path returns `NoCommunicationCert` error
- Users must import server certificate in Settings before license verification can succeed

---

## Migration Guide (For Users)

### If Upgrading to feature/license Branch

1. **Import Server Certificate**
   - Navigate to Settings → License
   - Click "Import Certificate"
   - Select server's communication certificate file (PEM or DER format)
   - Save settings

2. **Re-Import License File**
   - Navigate to Settings → License
   - Click "Import License File"
   - Select your license.dat file
   - Verification will now use the imported certificate's public key

3. **Verify License Status**
   - Check Settings → License Information
   - Should show "Valid" status with expiry date
   - If still showing "NoCommunicationCert", ensure certificate was saved in step 1

---

## File Changes Summary

| File | Type | Change |
|------|------|--------|
| src-tauri/src/license/error.rs | Modified | Added `NoCommunicationCert` variants |
| src-tauri/src/license/payload.rs | Modified | Signature now takes `&RsaPublicKey` parameter |
| src-tauri/src/license/mod.rs | Modified | Added `extract_public_key_from_cert()`, updated `is_licensed()` |
| src-tauri/src/commands/license.rs | Modified | Updated to pass cert path, handle new error variant |
| src-tauri/src/lib.rs | Modified | Read cert path from settings before calling `is_licensed()` |
| docs/system-architecture.md | Created | Full architecture documentation |
| docs/code-standards.md | Created | Code organization and standards |
| docs/project-overview-pdr.md | Created | Requirements and design decisions |
| docs/project-changelog.md | Created | This file |

---

## Testing Changes

**New Test Cases Added:**
- X.509 certificate parsing (PEM format)
- X.509 certificate parsing (DER format)
- RSA public key extraction from certificates
- Path traversal validation (reject relative paths)
- Path traversal validation (reject `..` segments)
- Missing certificate file handling
- Corrupt certificate file handling
- Full integration: import_license_file with cert verification

**All Existing Tests:** Passing (verified on feature/license branch)

---

## Dependency Updates

No new dependencies added. Uses existing:
- `rsa` — RSA cryptography
- `x509-parser` — X.509 certificate parsing
- `sha2` — SHA256 hashing
- `base64` — Base64 encoding/decoding
- `serde_json` — JSON serialization

---

## Next Steps

1. **Code Review** — Verify signature verification refactor on peer review
2. **QA Testing** — Test with real Windows tokens and production certificates
3. **Documentation Review** — Update help docs with certificate import instructions
4. **Merge to Main** — After approval, merge feature/license to main
5. **Release** — Create tagged release with changelog

---

## Versioning Strategy

Following semantic versioning:
- **Major:** Breaking changes to Tauri command API or license format
- **Minor:** New features (e.g., multi-token support, CRL integration)
- **Patch:** Bug fixes and security patches

---

## Contributors

- License signature refactor (runtime cert extraction): Development team
- Two-factor licensing system (initial): Development team
- Credential export alignment: Development team
