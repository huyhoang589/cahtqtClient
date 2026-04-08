# Codebase Summary

**Last Updated:** 2026-04-07  
**Repository:** CAHTQT Client (Tauri Desktop App)  
**Language:** Rust (backend) + TypeScript (frontend)  
**Total Files:** 50  
**Total Code Tokens:** ~117k  

---

## Directory Structure

```
cahtqt-client/
├── src-tauri/                          # Rust backend (Tauri commands & logic)
│   ├── src/
│   │   ├── lib.rs                      # App initialization, AppState, startup hook
│   │   ├── app_log.rs                  # Logging/telemetry emission to frontend
│   │   ├── db/                         # SQLite database access
│   │   │   ├── mod.rs
│   │   │   ├── settings_repo.rs        # Settings CRUD (PKCS#11 path, cert path)
│   │   │   └── ...
│   │   ├── commands/                   # Tauri IPC commands
│   │   │   ├── license.rs              # License verification & credential export
│   │   │   ├── encrypt.rs              # Hybrid RSA/AES encryption
│   │   │   ├── etoken.rs               # Token utilities
│   │   │   └── ...
│   │   ├── etoken/                     # PKCS#11 token management
│   │   │   ├── library_detector.rs     # Auto-detect Windows PKCS#11 library
│   │   │   ├── token_manager.rs        # Session/slot management
│   │   │   ├── certificate_reader.rs   # X.509 cert extraction from tokens
│   │   │   └── ...
│   │   ├── license/                    # 2F-HBLS license verification
│   │   │   ├── mod.rs                  # Main pipeline (is_licensed, extract_public_key_from_cert)
│   │   │   ├── error.rs                # LicenseStatus, LicenseError enums
│   │   │   ├── payload.rs              # License file I/O & RSA verification
│   │   │   ├── machine.rs              # Hardware fingerprinting
│   │   │   ├── token.rs                # PKCS#11 token operations
│   │   │   └── ...
│   │   └── ...
│   ├── Cargo.toml                      # Rust dependencies
│   ├── Cargo.lock                      # Locked dependency versions
│   ├── tauri.conf.json                 # Tauri configuration
│   └── build.rs                        # Build script (capability setup)
│
├── src/                                # TypeScript/React frontend
│   ├── types/                          # Type definitions for Tauri commands
│   │   ├── index.ts
│   │   └── ...
│   ├── components/                     # React components
│   │   ├── LicenseGate.tsx             # License check gate
│   │   ├── LicenseSection.tsx          # Settings license UI
│   │   └── ...
│   ├── commands/                       # Tauri command wrappers
│   │   ├── index.ts
│   │   └── ...
│   ├── pages/                          # Route pages
│   ├── App.tsx                         # Root component
│   ├── main.tsx                        # Entry point
│   └── ...
│
├── docs/                               # Documentation (newly created)
│   ├── system-architecture.md          # License verification pipeline & components
│   ├── code-standards.md               # Rust/TypeScript conventions
│   ├── project-overview-pdr.md         # Requirements & design decisions
│   ├── project-changelog.md            # Feature history & breaking changes
│   ├── codebase-summary.md             # This file
│   └── journals/                       # Development notes
│
├── plans/                              # Implementation plans
│   ├── 260407-1947-license-comm-cert-verification/
│   │   ├── plan.md
│   │   ├── phase-01-*.md
│   │   └── reports/
│   └── ...
│
├── .github/                            # GitHub config (CI/CD)
├── README.md                           # Project overview
├── CLAUDE.md                           # Development rules
└── .gitignore
```

---

## Key Modules

### License Module (`src/license/`)

**Purpose:** Two-factor hardware-bound license system verification.

| File | Responsibility | Key Functions |
|------|---|---|
| mod.rs | Main verification pipeline | `is_licensed()`, `extract_public_key_from_cert()` |
| error.rs | Error types and serialization | `LicenseError`, `LicenseStatus`, `LicenseInfo` |
| payload.rs | License file I/O and signature verification | `read_license_file()`, `verify_license_signature()`, `parse_license_payload()` |
| machine.rs | Hardware fingerprinting | `get_machine_fingerprint()`, `get_cpu_id()`, `get_board_serial()` |
| token.rs | PKCS#11 token operations | `get_token_serial()`, `verify_token_challenge()` |

**Flow:**
```
is_licensed(pkcs11_path, app_data_dir, comm_cert_path)
  ├─ Phase A: Token verification
  │  ├─ Initialize PKCS#11
  │  ├─ Get token serial
  │  └─ Challenge-response
  └─ Phase B: License binding
     ├─ Read license.dat
     ├─ Extract public key from cert
     ├─ Verify signature
     ├─ Parse payload
     └─ Validate bindings (fingerprint, serial, expiry)
```

### EToken Module (`src/etoken/`)

**Purpose:** PKCS#11 token communication and certificate extraction.

| File | Responsibility |
|------|---|
| library_detector.rs | Auto-detect PKCS#11 library on Windows |
| token_manager.rs | Session/slot management, token info queries |
| certificate_reader.rs | Extract certificates and attributes from tokens |

**Usage:** Called by license verification and credential export commands.

### Database Module (`src/db/`)

**Purpose:** SQLite settings persistence.

| File | Responsibility |
|------|---|
| settings_repo.rs | CRUD operations for settings key-value store |

**Key Settings:**
- `pkcs11_mode` (auto/manual)
- `pkcs11_manual_path`
- `communication_cert_path`
- `output_data_dir`

### Commands Module (`src/commands/`)

**Purpose:** Tauri IPC endpoints exposed to frontend.

| File | Commands |
|------|---|
| license.rs | `check_license()`, `get_license_info()`, `export_machine_credential()`, `import_license_file()` |
| encrypt.rs | `encrypt_message()`, `decrypt_message()` |
| etoken.rs | Token utility commands |

**Pattern:** All commands are async, return serialized types, validate inputs.

---

## License Verification Pipeline

### Request Flow
```
Frontend
  ↓
invoke("check_license")
  ↓
Tauri command: check_license()
  ├─ In debug: Return "ok" (bypass)
  └─ In release:
       ├─ Read cached LicenseInfo from AppState
       ├─ Map status to response
       └─ Return to frontend
  ↓
Frontend: LicenseGate routes based on status
  ├─ Valid → Show app
  ├─ NoToken / NotFound → Show message
  └─ Error → Show error page
```

### Initialization (Startup Hook in lib.rs)
```
App startup
  ↓
Initialize PKCS#11 library (from settings or auto-detect)
  ↓
Read comm_cert_path from database
  ↓
Call license::is_licensed(pkcs11_path, app_data_dir, comm_cert_path)
  ├─ Phase A: Token verification
  ├─ Phase B: License signature verification
  └─ Return LicenseInfo
  ↓
Cache in AppState::license_info (Mutex)
  ↓
App ready
```

---

## Data Types

### LicenseInfo
```rust
pub struct LicenseInfo {
    pub status: LicenseStatus,
    pub expires_at: Option<i64>,  // Unix timestamp
    pub product: Option<String>,
}
```

### LicenseStatus (enum)
- `Valid` — All checks passed
- `Expired` — License timestamp exceeded
- `NotFound` — No license.dat file
- `NoToken` — Token not inserted or PKCS#11 failed
- `TokenMismatch` — License bound to different token
- `MachineMismatch` — License bound to different hardware
- `Corrupted` — Invalid file/signature/JSON
- `NoCommunicationCert` — Cert path not configured or missing

### MachineCredential (JSON)
```json
{
  "board_serial": "XXXXXXXX",
  "cpu_id": "XXXXXXXX",
  "token_serial": "XXXXXXXX",
  "user_name": "CN from certificate",
  "registered_at": "2026-04-07"
}
```

---

## Dependency Map

### Cryptography
- `rsa` — RSA signature verification (PKCS1v15-SHA256)
- `sha2` — SHA256 hashing (machine fingerprint)
- `base64` — License file encoding

### X.509 & PKCS#11
- `x509-parser` — X.509 certificate parsing
- `pkcs11` — PKCS#11 FFI bindings

### Serialization
- `serde` + `serde_json` — JSON serialization
- `sqlx` — SQLite queries

### Framework
- `tauri` — Desktop framework
- `tokio` — Async runtime

### Utilities
- `chrono` — Unix timestamps
- `uuid` — Unique identifiers

---

## Security Considerations

### Implemented Controls

1. **Signature Verification:** RSA-PKCS1v15-SHA256 over license payload
2. **Path Safety:** Reject relative paths and `..` traversal
3. **Token Ownership:** Challenge-response proves private key possession
4. **Machine Binding:** SHA256 hash of hardware IDs
5. **Error Sanitization:** No keys, paths, or serials in error messages
6. **Certificate Validation:** X.509 parsing enforces structure

### Deployment Security

- Communication certificate path stored in SQLite (not hardcoded)
- PKCS#11 library path from settings (supports different platforms/installations)
- License file path validated before reading
- All file I/O behind path safety checks

---

## Testing Coverage

### Unit Tests

**License Module:**
- License file parsing (Base64, separator)
- RSA signature verification
- Certificate parsing (PEM/DER)
- Public key extraction
- Error handling

**Machine Module:**
- Hardware fingerprinting
- CPU ID and Board Serial retrieval

**EToken Module:**
- Token detection and initialization
- Certificate extraction

### Integration Tests

- Full license verification pipeline
- Token challenge-response
- Settings integration
- Tauri command flow

---

## Build & Deployment

### Build Process

```bash
cd src-tauri
cargo build --release
```

Outputs Windows executable in `src-tauri/target/release/`.

### Configuration

**tauri.conf.json:**
- Window settings (size, icons)
- Capabilities (file access, OS commands)
- Build/dev URLs

**Cargo.toml:**
- Rust dependencies and versions
- Target triple configuration

### Database

SQLite database created at `app_data_dir/cahtqt.db` on first run.

---

## File Statistics

| Category | Files | LOC | Tokens |
|----------|-------|-----|--------|
| Schemas (JSON) | 3 | ~500 | ~72k |
| Source Code (Rust) | 25 | ~3000 | ~35k |
| Commands (Rust) | 5 | ~1000 | ~10k |
| Frontend (TS/TSX) | 10 | ~2000 | ~8k |
| Config/Build | 5 | ~500 | ~2k |

---

## Recent Changes (feature/license branch)

### License Module Refactor

**Changed:**
- `verify_license_signature()` now takes `&RsaPublicKey` parameter
- `is_licensed()` accepts `comm_cert_path: Option<&str>` from settings
- Added `extract_public_key_from_cert()` for X.509 parsing
- Removed hardcoded key constant (was behind `compile_error!`)

**Impact:**
- Supports certificate rotation without recompilation
- Requires comm_cert_path in settings (no fallback)
- All builds verify signatures (no debug bypass)

---

## Module Dependencies

```
lib.rs (initialization)
├── license (verification pipeline)
│   ├── error (types)
│   ├── payload (file I/O)
│   ├── machine (fingerprinting)
│   └── token (PKCS#11 ops)
├── etoken (token management)
│   ├── library_detector
│   ├── token_manager
│   └── certificate_reader
├── db (settings)
└── commands (IPC)
    ├── license
    ├── encrypt
    └── etoken
```

---

## Quick Reference

### Adding a New Command

1. Define Rust function in `src/commands/module.rs`
2. Add `#[tauri::command]` attribute
3. Define TypeScript response type in `src/types/`
4. Create wrapper in `src/commands/` (TS)
5. Call from component: `invoke("command_name", args)`

### Updating License Verification

1. Modify logic in `src/license/mod.rs` (pipeline)
2. Update error types in `src/license/error.rs` if needed
3. Update `commands/license.rs` to handle new errors
4. Test with real certificates

### Debugging PKCS#11 Issues

1. Check `pkcs11_mode` setting (auto/manual)
2. Verify library path in settings
3. Check token insertion (physical connection)
4. Review `app_log` for PKCS#11 init errors

---

## Performance Notes

- License verification at startup: ~500ms (PKCS#11 init + signature check)
- Subsequent checks: < 1ms (cached in AppState)
- Certificate parsing: < 100ms
- No runtime key generation (only parsing)

---

## Known Limitations

1. Single token support (multi-token untested)
2. No certificate revocation checking (CRL/OCSP)
3. Challenge-response may skip if PIN required
4. No system time validation against cert validity period
