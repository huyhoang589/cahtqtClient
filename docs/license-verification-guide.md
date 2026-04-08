# License Verification Implementation Guide

**Purpose:** Technical reference for license signature verification refactor and integration points.

---

## Overview

License verification follows a two-phase approach:
- **Phase A:** Token presence verification via PKCS#11
- **Phase B:** License file integrity via RSA signature verification

The refactor moves from hardcoded public key to **runtime extraction from X.509 certificate**, enabling certificate rotation without code changes.

---

## Phase A: Token Verification

### Purpose
Proves that a valid token is present and holds the private key matching the public key in user credentials.

### Process

**Step 1: Initialize PKCS#11**
```rust
let pkcs11 = crate::etoken::token_manager::initialize(pkcs11_lib_path)
    .map_err(|e| LicenseError::TokenMissing(e))?;
```

- Loads PKCS#11 library from path (auto-detected or manual setting)
- Returns error if library not found or initialization fails
- Typical duration: ~200ms on Windows with eToken

**Step 2: Get Token Serial**
```rust
let token_serial = token::get_token_serial(&pkcs11)?;
```

- Queries token info via PKCS#11 API
- Returns token's unique serial number
- Used for license binding (ensure license matches inserted token)

**Step 3: Challenge-Response**
```rust
let machine_fp = machine::get_machine_fingerprint();
let _ = token::verify_token_challenge(&session, &machine_fp);
```

- Machine fingerprint = SHA256(CPU ID + Board Serial + MAC addresses)
- Sign fingerprint with token's private key
- Verify signature with token's public key (proves token holds matching key)
- Result is best-effort: Some tokens require PIN for C_Sign, continues without it

### Error Handling

| Error | Meaning | Recovery |
|-------|---------|----------|
| `TokenMissing` | PKCS#11 init failed or no token inserted | User inserts token or fixes library path |
| `TokenMissing` | No slots with token available | User inserts token |
| No error | Challenge-response fails silently | Proceeds to Phase B (token proves itself via license binding) |

---

## Phase B: License File Verification

### Purpose
Verifies that the license.dat file is valid and signed by the server's private key.

### File Format

```
[Base64-encoded data]

When Base64-decoded:
[JSON payload bytes]||SIG||[RSA signature bytes]
```

**Example flow:**
1. Read license.dat as raw bytes
2. Base64-decode to get: `payload || ||SIG|| || signature`
3. Split by `||SIG||` separator
4. Payload = JSON license info
5. Signature = RSA bytes (from server's private key)

### Process

**Step 1: Read License File**
```rust
let (payload_bytes, sig_bytes) = payload::read_license_file(app_data_dir)?;
```

**Step 2: Get Communication Certificate Path**
```rust
let comm_path = comm_cert_path
    .filter(|p| !p.is_empty())
    .ok_or(LicenseError::NoCommunicationCert)?;
```

- Path comes from SQLite settings table
- Must be non-empty string
- Returns `NoCommunicationCert` if not configured

**Step 3: Validate Certificate Path**
```rust
let comm_path_obj = std::path::Path::new(comm_path);
if !comm_path_obj.is_absolute() || comm_path.contains("..") {
    return Err(LicenseError::InvalidKey("Invalid communication cert path".into()));
}
if !comm_path_obj.exists() {
    return Err(LicenseError::NoCommunicationCert);
}
```

**Security checks:**
- Reject relative paths (must be absolute)
- Reject `..` directory traversal attempts
- Verify file exists before attempting to read

**Step 4: Read Certificate File**
```rust
let cert_data = std::fs::read(comm_path)
    .map_err(|e| LicenseError::InvalidKey(format!("Cannot read communication cert: {}", e)))?;
```

**Step 5: Extract Public Key from Certificate**
```rust
let public_key = extract_public_key_from_cert(&cert_data)?;
```

**Certificate Parsing Logic:**
```rust
fn extract_public_key_from_cert(cert_data: &[u8]) -> Result<RsaPublicKey, LicenseError> {
    // Auto-detect PEM vs DER
    let der_bytes: Vec<u8> = if cert_data.windows(b"-----BEGIN".len()).any(|w| w == b"-----BEGIN") {
        // PEM format: Base64-encoded, extract binary
        let (_, pem) = x509_parser::pem::parse_x509_pem(cert_data)
            .map_err(|e| LicenseError::InvalidKey(format!("PEM parse error: {:?}", e)))?;
        pem.contents
    } else {
        // DER format: raw binary
        cert_data.to_vec()
    };

    // Parse X.509 certificate
    let (_, cert) = parse_x509_certificate(&der_bytes)
        .map_err(|e| LicenseError::InvalidKey(format!("Certificate parse error: {:?}", e)))?;

    // Extract SPKI DER (SubjectPublicKeyInfo)
    let spki_der = cert.public_key().raw.to_vec();
    
    // Decode RSA public key from SPKI
    RsaPublicKey::from_public_key_der(&spki_der)
        .map_err(|e| LicenseError::InvalidKey(format!("Not an RSA certificate: {}", e)))
}
```

**Key Points:**
- Supports both PEM (text) and DER (binary) formats
- Auto-detects format by looking for `-----BEGIN` magic bytes
- Parses X.509 structure to validate certificate
- Extracts SPKI (Subject Public Key Info) in DER format
- Converts to `RsaPublicKey` struct
- Returns error if certificate is not RSA

**Step 6: Verify RSA Signature**
```rust
payload::verify_license_signature(&payload_bytes, &sig_bytes, &public_key)?;
```

**Signature Verification Logic:**
```rust
pub fn verify_license_signature(
    payload: &[u8],
    sig: &[u8],
    public_key: &RsaPublicKey,
) -> Result<(), LicenseError> {
    // Create verifying key with SHA256 hasher
    let verifying_key = VerifyingKey::<Sha256>::new(public_key.clone());
    
    // Convert raw signature bytes to Signature struct
    let signature = rsa::pkcs1v15::Signature::try_from(sig)
        .map_err(|e| LicenseError::InvalidKey(format!("Invalid signature format: {}", e)))?;

    // Verify: payload was signed by matching private key
    verifying_key
        .verify(payload, &signature)
        .map_err(|_| LicenseError::Corrupted("RSA signature verification failed".into()))
}
```

**Algorithm:** RSA-PKCS1v15 with SHA256 hash function

**Step 7: Parse License Payload**
```rust
let license = payload::parse_license_payload(&payload_bytes)?;
```

**Payload Structure:**
```json
{
  "product": "CAHTQT Client",
  "machine_fp": "sha256_hash_of_hardware_ids",
  "token_serial": "XXXXXXXX",
  "issued_at": 1712534400,
  "expires_at": 1744070400,
  "version": "1.0"
}
```

**Fields:**
- `product` — Application name (informational)
- `machine_fp` — SHA256 hash of hardware IDs (optional, if present must match)
- `token_serial` — Token serial (optional, if present must match current token)
- `issued_at` — Unix timestamp (informational)
- `expires_at` — Unix timestamp (optional, if present must be >= now)
- `version` — License format version

**Step 8: Validate Machine Binding**
```rust
if let Some(ref licensed_fp) = license.machine_fp {
    if *licensed_fp != machine_fp {
        return Err(LicenseError::MachineMismatch);
    }
}
```

**Machine Fingerprint Calculation:**
```
SHA256(cpu_id + board_serial + concat(sorted_mac_addresses))
```

- Computed at runtime from Windows WMI/Registry
- If license includes fingerprint, must match exactly
- If fingerprint mismatch, license not valid on this machine

**Step 9: Validate Token Binding**
```rust
if let Some(ref licensed_serial) = license.token_serial {
    if *licensed_serial != token_serial {
        return Err(LicenseError::TokenMismatch);
    }
}
```

- If license specifies token serial, current token must match
- Token must be inserted from Step 2

**Step 10: Validate Expiry**
```rust
if let Some(expires_at) = license.expires_at {
    let now = chrono::Utc::now().timestamp();
    if now > expires_at {
        return Err(LicenseError::Expired);
    }
}
```

- If license specifies expiry, must be in future
- Uses UTC time (no timezone handling)

---

## Integration Points

### Startup Hook (lib.rs)

```rust
// In app initialization:
let pkcs11_path = /* from settings or auto-detect */;
let app_data_dir = /* app data directory */;
let comm_cert_path = /* from SQLite settings */;

let license_info = license::is_licensed(&pkcs11_path, &app_data_dir, comm_cert_path.as_deref());

// Cache in AppState
AppState {
    license_info: Mutex::new(license_info),
    // ... other state
}
```

### Tauri Commands (commands/license.rs)

**check_license():**
- Called by LicenseGate component on app mount
- Returns cached `LicenseInfo` from AppState
- Serializes `LicenseStatus` to frontend

**get_license_info():**
- Called by Settings component
- Returns full license details (status, expiry, product)
- Used for diagnostics

**import_license_file():**
- User selects license.dat file
- Validates file structure (Base64 + separator)
- Copies to app_data_dir/license.dat
- Re-runs full verification pipeline
- Updates cached state
- Returns new status to frontend

**export_machine_credential():**
- Collects hardware IDs + token serial
- Reads user certificate CN from token
- Serializes to JSON
- Saves to user-selected directory

### Settings Integration (SQLite)

**Read comm_cert_path:**
```rust
let comm_cert_path = crate::db::settings_repo::get_setting(&state.db, "communication_cert_path")
    .await
    .ok()
    .flatten();  // Option<String>
```

**Write comm_cert_path:**
```rust
crate::db::settings_repo::set_setting(&state.db, "communication_cert_path", cert_path).await?;
```

---

## Error Scenarios & Recovery

| Scenario | Error | User Action |
|----------|-------|-------------|
| Token not inserted | `NoToken` | Insert token |
| PKCS#11 library not found | `TokenMissing` | Configure library path in Settings |
| License file missing | `NotFound` | Import license.dat via Settings |
| License file corrupted (bad Base64) | `Corrupted` | Re-export license from server |
| License file corrupted (missing separator) | `Corrupted` | Re-export license from server |
| License file corrupted (bad JSON) | `Corrupted` | Re-export license from server |
| Signature verification fails | `Corrupted` | License file tampered, re-export from server |
| Cert path not configured | `NoCommunicationCert` | Import server certificate in Settings |
| Cert file missing | `NoCommunicationCert` | Check certificate file still exists |
| Cert is not RSA | `InvalidKey` | Use correct server certificate (must be RSA) |
| Machine fingerprint mismatch | `MachineMismatch` | Contact IT for hardware-specific license |
| Token serial mismatch | `TokenMismatch` | Insert correct token |
| License expired | `Expired` | Contact IT for license renewal |

---

## Testing Checklist

### Unit Tests

- [ ] Base64 decode and separator split
- [ ] RSA signature verification (valid, invalid, tampered)
- [ ] X.509 parsing (PEM format)
- [ ] X.509 parsing (DER format)
- [ ] X.509 parsing (invalid certificate)
- [ ] Public key extraction from certificate
- [ ] Public key extraction (non-RSA certificate)
- [ ] Path validation (absolute path)
- [ ] Path validation (relative path rejected)
- [ ] Path validation (`..` directory traversal rejected)
- [ ] Path validation (missing file)

### Integration Tests

- [ ] Full pipeline with real token
- [ ] Full pipeline with test certificate
- [ ] Error handling: missing cert path
- [ ] Error handling: corrupt license file
- [ ] Error handling: bad signature
- [ ] Settings integration: save/load cert path
- [ ] Command flow: import_license_file with verification

### Manual Tests

- [ ] Windows auto-detect PKCS#11 library
- [ ] Token inserted during startup
- [ ] Token removed after startup (continues running)
- [ ] Certificate import from file
- [ ] License import with signature verification
- [ ] License check returns "Valid" status
- [ ] Settings display license expiry date

---

## Debugging Tips

### Enable Verbose Logging

Check app log in Settings → Logs for PKCS#11 init errors and certificate parsing issues.

### Verify Certificate Format

```bash
# Check certificate is PEM or DER
file server-cert.pem
# or
openssl x509 -in server-cert.pem -text -noout
```

### Verify RSA Key Size

```bash
openssl x509 -in server-cert.pem -text -noout | grep "Public-Key:"
# Should show: Public-Key: (2048 bit) or (4096 bit)
```

### Check License File Format

```bash
# Verify Base64 encoding
base64 -d license.dat | xxd | head
# Should show binary data with ||SIG|| separator visible
```

### Test Signature Verification Offline

Use `openssl` to verify license signature with public key:
```bash
# Extract public key from certificate
openssl x509 -in server-cert.pem -pubkey -noout > pubkey.pem

# Verify signature (if signature is detached/separate)
openssl dgst -sha256 -verify pubkey.pem -signature license.sig license.payload
```

---

## Future Enhancements

### Certificate Pinning

Hard-code public key hash after first deployment:
```rust
const SERVER_KEY_FINGERPRINT: &str = "sha256:XXXXXXXX...";

// In extract_public_key_from_cert():
let key_hash = compute_sha256(&spki_der);
if key_hash != SERVER_KEY_FINGERPRINT {
    return Err(LicenseError::InvalidKey("Key mismatch".into()));
}
```

### Revocation Checking

Integrate CRL (Certificate Revocation List):
```rust
let crl = fetch_crl(&cert.issuer_url)?;
if crl.is_revoked(&cert.serial) {
    return Err(LicenseError::InvalidKey("Certificate revoked".into()));
}
```

### Time Synchronization Validation

Reject licenses if system clock is too far off:
```rust
let cert_validity_start = cert.validity.not_before.timestamp();
let now = chrono::Utc::now().timestamp();
if (now - cert_validity_start).abs() > 86400 {  // 1 day tolerance
    return Err(LicenseError::InvalidKey("System clock skewed".into()));
}
```

---

## Key Takeaways

1. **Two-Phase Verification:** Phase A proves token presence, Phase B proves license integrity
2. **Runtime Certificate Extraction:** Public key derived from configurable certificate (no hardcoding)
3. **Path Safety:** All certificate paths validated for security
4. **Error Propagation:** Detailed errors logged, sanitized messages to frontend
5. **Caching:** Verification runs once at startup, result cached for fast checks
6. **Extensibility:** Design supports certificate rotation, revocation, and future enhancements
