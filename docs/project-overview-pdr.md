# CAHTQT Client — Project Overview & PDR

## Executive Summary

**CAHTQT Client** is a secure desktop application enabling certificate-based authentication and encrypted communication through hardware security tokens (eTokens). The system enforces mandatory license verification at startup using two-factor hardware-bound licensing (2F-HBLS) with RSA signature verification against a server-provided certificate.

**Current Phase:** License Signature Verification Refactor Complete

---

## Functional Requirements

### FR-1: Hardware Token Integration (PKCS#11)

- Initialize PKCS#11 library (auto-detect or manual path)
- Query token info (serial, status)
- Read user certificates from token
- Perform challenge-response (prove private key ownership)
- Support multi-token scenarios (use first available)

**Status:** IMPLEMENTED (commit 1e1f7f2)

### FR-2: License Verification (2F-HBLS)

- **Phase A:** Token presence via PKCS#11
  - Get token serial
  - Challenge-response: Sign machine fingerprint with token
  - Result: Proves token holds matching private key

- **Phase B:** License file binding
  - Read license.dat from app_data_dir
  - Extract RSA public key from communication certificate
  - Verify signature: RSA-PKCS1v15-SHA256
  - Parse JSON payload: product, machine_fp, token_serial, issued_at, expires_at
  - Validate bindings: fingerprint match, token match, expiry check
  - Cache result in AppState

**Status:** IMPLEMENTED (commit 1e1f7f2); Refactored (feature/license branch)

### FR-3: Runtime Certificate-Based Signature Verification

- Accept communication certificate path from settings (SQLite)
- Auto-detect PEM or DER format
- Parse X.509 certificate structure
- Extract RSA public key (SPKI DER)
- Reject relative paths, `..` directory traversal
- Handle missing/corrupt certificates gracefully

**Status:** IMPLEMENTED (feature/license branch)

### FR-4: Machine Credential Export

- Collect hardware IDs: CPU ID, Board Serial, Token Serial
- Extract user identity from token certificate (CN)
- Serialize to JSON matching server spec
- Save to user-selected output directory
- Timestamp files for disambiguation

**Status:** IMPLEMENTED (commit af5bf9a, 464c510)

### FR-5: Secure Credential Storage

- Store PKCS#11 path in settings database
- Store communication certificate path in settings database
- Store output directory preference
- No plaintext key storage (only paths to certificates)

**Status:** IMPLEMENTED (Settings commands)

---

## Non-Functional Requirements

### NFR-1: Security

| Requirement | Implementation |
|-------------|---|
| No hardcoded secrets | Server public key extracted at runtime from configurable cert |
| Path traversal protection | Validate absolute paths, reject `..` segments |
| Error message sanitization | No keys, paths, or serials in frontend error text |
| RSA signature verification | PKCS1v15-SHA256 per spec |
| Token ownership proof | Challenge-response with private key |
| Machine binding | SHA256 hash of hardware IDs |

**Compliance:** PKIX standards (X.509, PKCS#1)

### NFR-2: Performance

- License verification at startup (cached in AppState)
- Token operations non-blocking (async Tauri commands)
- Certificate parsing < 100ms
- No runtime key generation (only parsing)

### NFR-3: Usability

- Auto-detect PKCS#11 library on Windows
- Clear error messages for common issues
- Settings UI for manual configuration
- One-click credential export

### NFR-4: Maintainability

- Modular license pipeline (Phase A, Phase B)
- Comprehensive error types with Display impl
- Doc comments on all public functions
- Test suite for crypto operations

---

## Architecture Decisions

### AD-1: Runtime Public Key Extraction (vs. Hardcoded Key)

**Decision:** Extract RSA public key from communication certificate at runtime.

**Rationale:**
- Supports key rotation without recompilation
- Allows certificate updates in the field
- Reduces attack surface (no embedded secrets)
- Simplifies deployment (no manual key insertion in source)

**Tradeoff:** Requires certificate path configuration (stored in SQLite settings)

### AD-2: X.509 Certificate Format Support

**Decision:** Support both PEM and DER formats with auto-detection.

**Rationale:**
- PEM (Base64-encoded text) easier for human inspection
- DER (binary) more efficient for transmission
- Auto-detection simplifies user experience (no format selection needed)

### AD-3: Async License Verification at Startup

**Decision:** Run full license verification (Phase A + B) in startup hook before AppState initialization.

**Rationale:**
- Catch licensing issues before app is usable
- Block unauthorized usage early (security)
- Result cached for fast frontend checks

**Tradeoff:** Extends startup time (~500ms for PKCS#11 init + signature verification)

### AD-4: Path Safety Validation

**Decision:** Reject relative paths and `..` directory traversal on certificate and credential paths.

**Rationale:**
- Prevents directory traversal attacks
- Enforces explicit full paths (user intention clear)
- Compliant with secure coding standards

---

## Acceptance Criteria

### AC-1: License Signature Verification

- [ ] `verify_license_signature()` takes `&RsaPublicKey` parameter (no hardcoded keys)
- [ ] `is_licensed()` accepts `comm_cert_path: Option<&str>` from settings
- [ ] Public key extracted from X.509 cert (PEM/DER auto-detect)
- [ ] Path traversal validation (absolute path, no `..`)
- [ ] `NoCommunicationCert` error returned if path not configured
- [ ] All builds verify signatures (no debug-build bypass)
- [ ] Tests pass for PEM, DER, missing, and corrupt certificates

### AC-2: Error Handling

- [ ] `LicenseStatus::NoCommunicationCert` variant added
- [ ] `LicenseError::NoCommunicationCert` variant added
- [ ] User-facing error message: "Communication certificate not configured. Please import the server certificate in Settings."
- [ ] No sensitive data in error messages

### AC-3: Integration with Commands

- [ ] `check_license()` command reflects `NoCommunicationCert` status
- [ ] `import_license_file()` re-runs full verification with comm cert
- [ ] Settings read `communication_cert_path` on startup
- [ ] Frontend receives properly mapped error messages

### AC-4: Documentation

- [ ] System architecture document updated
- [ ] Code standards reflect new function signatures
- [ ] Comments explain public key extraction flow
- [ ] No stale references to hardcoded keys

---

## Implementation Timeline

| Phase | Feature | Status | Commit |
|-------|---------|--------|--------|
| 1 | 2F-HBLS (token + license file) | DONE | 1e1f7f2 |
| 2 | Credential export (align to server spec) | DONE | af5bf9a |
| 3 | Add user_name (cert CN) to credential | DONE | 464c510 |
| 4 | License signature refactor (runtime cert extraction) | DONE | feature/license |

**Current Status:** All phases complete on feature/license branch. Ready for merge to main.

---

## Known Limitations & Future Work

### Current Limitations

1. **Single Token:** Code assumes first available token. Multi-token scenarios not fully tested.
2. **No CRL/OCSP:** No certificate revocation checking. Relies on expiry in license file.
3. **No Time Sync Validation:** Doesn't check system clock against NTP or cert validity period.
4. **PIN Handling:** Challenge-response may fail if token requires PIN (best-effort, proceeds without it).

### Future Enhancements

1. **Certificate Pinning:** Hard-code public key hash after deployment (prevent MITM).
2. **Revocation List:** Integrate CRL or OCSP for real-time key rotation.
3. **Multi-Token Support:** Allow user to select which token to use if multiple present.
4. **Offline Licensing:** Cache verified licenses locally with expiry-based re-verification.
5. **Hardware Security:** Integrate TPM for additional hardware binding (if available).

---

## Deployment Checklist

- [ ] Communication certificate path set in production settings
- [ ] Server public key imported into production settings
- [ ] License file created and distributed to users
- [ ] Help documentation updated (how to import cert and license)
- [ ] Support process documented for "NoCommunicationCert" errors
- [ ] Rollback plan if certificate needs replacement

---

## Testing Summary

**Unit Tests:**
- ✓ License file Base64 decoding and structure validation
- ✓ RSA signature verification (happy path and tampering)
- ✓ X.509 certificate parsing (PEM and DER)
- ✓ Public key extraction from certificates
- ✓ Error handling for missing/corrupt files
- ✓ Path traversal validation

**Integration Tests:**
- ✓ Full license verification pipeline (Phase A + B)
- ✓ Token challenge-response
- ✓ Settings integration (read comm_cert_path)
- ✓ Tauri command flow (import_license_file → re-verify)

**Manual Tests:**
- ✓ Windows PKCS#11 auto-detection
- ✓ Token insertion/removal handling
- ✓ License file import with signature verification
- ✓ Error message display on frontend

---

## Glossary

| Term | Definition |
|------|---|
| 2F-HBLS | Two-Factor Hardware-Bound License System |
| PKCS#11 | Standard for cryptographic token interface |
| SPKI | Subject Public Key Info (X.509 format) |
| DER | Distinguished Encoding Rules (binary cert format) |
| PEM | Privacy-Enhanced Mail (Base64-encoded cert format) |
| Challenge-Response | Cryptographic protocol proving private key possession |
| Machine Fingerprint | SHA256 hash of CPU ID + Board Serial + MAC addresses |
| Licensing Error | Mismatch between device state and license bindings |
