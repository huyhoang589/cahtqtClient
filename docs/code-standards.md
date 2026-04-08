# Code Standards

## Rust Code Organization

### Module Structure

**License Module** (`src/license/`):
```
src/license/
├── mod.rs              # Main verification pipeline (is_licensed, extract_public_key_from_cert)
├── error.rs            # LicenseStatus, LicenseInfo, LicenseError enums + Display impl
├── payload.rs          # License file I/O (read_license_file, verify_license_signature, parse_license_payload)
├── machine.rs          # Hardware fingerprinting (get_machine_fingerprint, get_cpu_id, get_board_serial)
└── token.rs            # PKCS#11 operations (get_token_serial, verify_token_challenge)
```

**EToken Module** (`src/etoken/`):
```
src/etoken/
├── library_detector.rs # Windows PKCS#11 library auto-detection
├── token_manager.rs    # PKCS#11 session/slot management
└── certificate_reader.rs # X.509 certificate extraction from tokens
```

**Commands** (`src/commands/`):
```
src/commands/
├── license.rs          # Tauri commands: check_license, get_license_info, export_machine_credential, import_license_file
├── encrypt.rs          # Hybrid RSA/AES encryption
└── etoken.rs           # Token utilities exposed to frontend
```

### Naming Conventions

- **Modules:** snake_case (`license`, `etoken`, `token_manager`)
- **Functions:** snake_case (`verify_license_signature`, `get_machine_fingerprint`)
- **Constants:** UPPER_SNAKE_CASE (`SIG_SEPARATOR`, `SERVER_PUBLIC_KEY_PEM`)
- **Types:** PascalCase (`LicenseError`, `LicenseStatus`, `LicenseInfo`)
- **Private items:** Prefix with `_` when intentionally unused (e.g., `_state` in debug builds)

### Error Handling

**Pattern:** Use custom `Result<T, CustomError>` types:
```rust
pub enum LicenseError {
    TokenMissing(String),
    InvalidKey(String),
    // ...
    NoCommunicationCert,
}

impl fmt::Display for LicenseError { /* ... */ }

impl LicenseError {
    pub fn to_status(&self) -> LicenseStatus { /* ...  */ }
}
```

**Rules:**
- All errors implement `Display` for user-facing messages
- Convert to serializable enum (`LicenseStatus`) before frontend exposure
- No sensitive data (keys, paths, serials) in error messages
- Use descriptive context strings in error variants

### Safety & Security

**Path Validation:**
```rust
// ✓ CORRECT: Validate before file operations
let path = std::path::Path::new(comm_path);
if !path.is_absolute() || comm_path.contains("..") {
    return Err(LicenseError::InvalidKey("Invalid path".into()));
}
```

**Type Safety:**
- Use `Option<T>` for nullable values (no raw `null` pointers)
- Use `Result<T, E>` for fallible operations
- Prefer `?` operator over manual error handling

**Memory Safety:**
- Avoid `unsafe` blocks unless unavoidable (PKCS#11 FFI is exception)
- Use `Vec<u8>` for binary data
- Leverage Rust's borrow checker — no manual memory management

---

## TypeScript Code Organization

### Command Response Types

**Pattern:** All Tauri commands return serialized types:
```typescript
export interface LicenseCheckResult {
  state: "ok" | "no_token" | "no_license" | "error";
  error_msg?: string;
}

export interface MachineCredentialResult {
  saved_path: string;
}
```

**Rules:**
- Match Rust serialization (`#[serde(rename_all = "snake_case")]`)
- Use optional fields (`?`) for nullable responses
- Validate response shape on frontend before consumption

### Frontend Integration

**LicenseGate Component:**
- Calls `invoke("check_license")` on mount
- Routes to `/license-check` if not licensed
- Routes to `/settings` for license management

**Commands Module:**
- Wraps Tauri `invoke()` calls
- Handles serialization/deserialization
- Provides type-safe frontend API

---

## Database Schema

### Settings Table

| Column | Type | Purpose |
|--------|------|---------|
| `key` | TEXT PRIMARY KEY | Setting identifier |
| `value` | TEXT | JSON-encoded value |

**Key Settings:**
- `pkcs11_mode`: "auto" or "manual"
- `pkcs11_manual_path`: Path to PKCS#11 library (if manual)
- `communication_cert_path`: Path to server certificate (for license verification)
- `output_data_dir`: Directory for exported credentials

---

## Testing Strategy

### Unit Tests

**License Module:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_payload_and_sig() { /* ... */ }

    #[test]
    fn test_verify_license_signature() { /* ... */ }

    #[test]
    fn test_extract_public_key_from_cert_pem() { /* ... */ }

    #[test]
    fn test_extract_public_key_from_cert_der() { /* ... */ }
}
```

**Guidelines:**
- Test both happy path and error cases
- Mock PKCS#11 for token tests (use test certificates)
- Use fixtures for license files (Base64-encoded test data)

---

## Build Configuration

### Cargo.toml

**Key Dependencies:**
- `tauri` — Desktop framework
- `serde` / `serde_json` — Serialization
- `rsa` — RSA cryptography
- `x509-parser` — X.509 cert parsing
- `pkcs11` — PKCS#11 FFI bindings
- `sha2` / `base64` — Crypto utilities
- `chrono` — Timestamps
- `sqlx` — Database driver

### Feature Flags

- `debug_assertions` — Dev-only bypasses (license gate auto-pass in debug builds)

---

## Documentation Comments

**Rule:** All public functions have doc comments:
```rust
/// Verify RSA-PKCS1v15-SHA256 signature over payload using caller-provided public key.
///
/// # Arguments
/// * `payload` — Raw license payload bytes (JSON)
/// * `sig` — RSA signature bytes
/// * `public_key` — Public key from server certificate
///
/// # Errors
/// Returns `LicenseError::InvalidKey` if signature format invalid.
/// Returns `LicenseError::Corrupted` if verification fails.
pub fn verify_license_signature(
    payload: &[u8],
    sig: &[u8],
    public_key: &RsaPublicKey,
) -> Result<(), LicenseError> { /* ... */ }
```

---

## Pre-Commit Checklist

- [ ] No hardcoded secrets (keys, tokens, paths)
- [ ] All public functions documented
- [ ] Error messages user-friendly (no keys/paths)
- [ ] Path validation on all user-supplied file paths
- [ ] Tests pass: `cargo test`
- [ ] Build succeeds: `cargo build --release`
- [ ] No compiler warnings (clippy)

---

## Common Patterns

### Async/Await in Tauri Commands

```rust
#[tauri::command]
pub async fn check_license(state: State<'_, AppState>) -> Result<LicenseCheckResult, String> {
    let info = state.license_info.lock()
        .map_err(|_| "License state unavailable".to_string())?;
    // ...
    Ok(result)
}
```

### Settings Retrieval

```rust
let settings = crate::db::settings_repo::get_all_settings(&state.db).await?;
let settings_map: HashMap<String, String> =
    settings.into_iter().map(|s| (s.key, s.value)).collect();

let pkcs11_path = settings_map.get("pkcs11_manual_path").cloned().unwrap_or_default();
```

### Path Safety

```rust
let path = std::path::Path::new(user_input);
if !path.is_absolute() || user_input.contains("..") {
    return Err("Invalid path".into());
}
if !path.exists() {
    return Err("Path not found".into());
}
let data = std::fs::read(path)?;
```
