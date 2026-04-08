# System Architecture

## Overview

CAHTQT Client is a Tauri desktop application integrating hardware security tokens (eTokens) with multi-factor licensing and secure credential export. The system enforces license verification at startup and provides certificate-based cryptographic operations.

## Core Components

### 1. License Verification System (2F-HBLS)

**Purpose:** Multi-factor license binding to prevent unauthorized use.

**Architecture:**
- **Phase A - Token Verification**: PKCS#11 challenge-response proves token presence
- **Phase B - License Binding**: RSA signature verification + machine/token fingerprinting

**Key Responsibilities:**
- `license::is_licensed()` — Full verification pipeline (Phase A + B)
- `license::payload::verify_license_signature()` — RSA-PKCS1v15-SHA256 signature verification
- `license::payload::read_license_file()` — Parses Base64-encoded license.dat (format: payload||SIG||signature)
- `license::machine::get_machine_fingerprint()` — Hardware ID collection
- `license::token::get_token_serial()` — PKCS#11 token serial extraction

**Data Flow:**

```
Startup
  ↓
Phase A: Token Presence Check
  ├─ Initialize PKCS#11 library
  ├─ Get token serial (pkcs11.get_token_info)
  ├─ Challenge-response: Sign machine_fp with token's private key
  └─ Result: Proves token holds matching private key

Phase B: License File Verification
  ├─ Read license.dat from app_data_dir
  ├─ Extract public key from communication certificate
  │  ├─ Load cert from disk (PEM/DER auto-detect)
  │  ├─ Parse X.509 structure
  │  └─ Extract SPKI DER public key
  ├─ Verify RSA signature: payload signed with issuer's private key
  ├─ Parse license JSON: {product, machine_fp, token_serial, issued_at, expires_at}
  ├─ Check bindings:
  │  ├─ machine_fp (if present) must match current hardware
  │  ├─ token_serial (if present) must match current token
  │  └─ expires_at (if present) must be >= current UTC time
  └─ Result: LicenseInfo cached in AppState

LicenseGate Frontend
  └─ Calls check_license() → reads cached LicenseInfo → routes based on status
```

### 2. Communication Certificate Management

**Purpose:** Runtime RSA public key source for license signature verification (as of refactor).

**Key Change (Refactor):**
- **Before:** Hardcoded `SERVER_PUBLIC_KEY_PEM` placeholder with `compile_error!` guard (required manual key insertion)
- **After:** Dynamic extraction from configurable communication certificate path (stored in SQLite settings)

**New Components:**
- `extract_public_key_from_cert()` — Auto-detects PEM/DER, parses X.509, extracts RSA public key
- `verify_license_signature()` — Takes `&RsaPublicKey` parameter (injected from cert)
- `NoCommunicationCert` error variant — Returned if cert path not configured or file missing

**Validation:**
- Path safety: Rejects relative paths and `..` directory traversal
- File existence check before reading
- X.509 parsing validates cert structure
- RSA key type validation (rejects non-RSA certificates)

### 3. Machine Credential Export

**Purpose:** Collect hardware identifiers for server-side license binding.

**Workflow:**
1. Read hardware IDs: CPU ID, Board Serial
2. Read token serial from PKCS#11
3. Read user certificate CN (subject common name)
4. Serialize to JSON matching server spec
5. Save to user-selected output directory

**Format:**
```json
{
  "board_serial": "string",
  "cpu_id": "string",
  "token_serial": "string",
  "user_name": "string (certificate CN)",
  "registered_at": "YYYY-MM-DD"
}
```

### 4. Token Management (PKCS#11)

**Purpose:** Hardware token communication via PKCS#11 API.

**Key Modules:**
- `etoken::library_detector` — Auto-detect PKCS#11 library on Windows
- `etoken::token_manager` — Session/slot management
- `etoken::certificate_reader` — Extract certificates and attributes

**Security:**
- Read-only sessions for public operations (no PIN required)
- Challenge-response uses token's private key (proves ownership)
- Certificate CN used for user identity (no private key export)

### 5. Encrypted Communication

**Purpose:** Secure data transfer with end-to-end encryption.

**Modules:**
- `etoken::certificate_reader` — Public certificate extraction
- `commands::encrypt` — RSA/AES hybrid encryption
- Database-backed key storage for message encryption keys

---

## License Verification Timeline

| Step | Component | Responsibility |
|------|-----------|---|
| 1 | Startup hook | Initialize PKCS#11, call `license::is_licensed()` |
| 2 | Phase A | Verify token present, get serial, run challenge-response |
| 3 | Phase B | Read license.dat, extract public key from comm cert |
| 4 | Signature | Verify RSA signature with extracted public key |
| 5 | Payload | Parse JSON, validate machine/token/expiry bindings |
| 6 | Cache | Store LicenseInfo in AppState mutex |
| 7 | Frontend | LicenseGate reads cached status on mount |

---

## Error Handling

**LicenseStatus Enum** (serialized to frontend):
- `Valid` — All checks passed
- `Expired` — License timestamp exceeded
- `NotFound` — No license.dat file
- `NoToken` — Token not inserted or PKCS#11 init failed
- `TokenMismatch` — License bound to different token
- `MachineMismatch` — License bound to different hardware
- `Corrupted` — Invalid Base64, missing separator, bad JSON, bad signature
- `NoCommunicationCert` — Cert path not set or file missing

**User-Facing Messages:**
- Messages auto-generated from `LicenseError::Display` implementation
- No sensitive data exposed (keys, paths remain internal)

---

## Dependencies

- `rsa` — RSA signature verification (PKCS1v15-SHA256)
- `x509-parser` — X.509 certificate parsing
- `sha2` — SHA256 hashing
- `base64` — License file encoding
- `chrono` — Unix timestamp handling
- `serde_json` — License payload JSON

---

## Security Considerations

1. **Signature Verification:** RSA-PKCS1v15-SHA256, no hardcoded keys
2. **Path Safety:** Reject relative paths, `..` traversal on cert path
3. **Token Ownership:** Challenge-response proves private key possession
4. **Machine Binding:** SHA256(CPU ID + Board Serial + SHA256(all MAC addresses))
5. **Certificate Validation:** X.509 parsing enforces cert structure
6. **Error Messaging:** No keys, paths, or token serials in frontend error text

---

## Future Considerations

- Certificate pinning (hardcode public key hash after initial deployment)
- Revocation list (CRL/OCSP integration)
- Time synchronization validation (prevent clock skew attacks)
- Rate limiting on signature verification failures
