# Phase 3: Wire DB + Cert Extraction into is_licensed

## Context Links
- [mod.rs](../../src-tauri/src/license/mod.rs) — is_licensed(), verify_full()
- [lib.rs](../../src-tauri/src/lib.rs) — AppState, startup license check (line 114-131)
- [commands/license.rs](../../src-tauri/src/commands/license.rs) — import_license_file calls is_licensed (line 226)
- [settings_repo.rs](../../src-tauri/src/db/settings_repo.rs) — get_setting()
- [cert_parser.rs](../../src-tauri/src/cert_parser.rs) — parse_cert_file/parse_cert_bytes
- [callbacks.rs](../../src-tauri/src/htqt_ffi/callbacks.rs) — extract_spki_der pattern (line 228-231)
- [plan.md](./plan.md)

## Overview
- **Priority:** P1
- **Status:** Complete
- **Description:** Update `is_licensed()` + `verify_full()` to accept `SqlitePool`, read comm cert path from DB, extract RSA public key from X.509 cert, pass to `verify_license_signature()`. Update all callers. Remove debug-only signature skip.

## Key Insights
- `is_licensed()` currently sync — but DB access is async. Two options:
  - Make `is_licensed` async (cascading change)
  - Pass comm_cert_path as parameter (caller reads from DB before calling)
  - **Decision: Pass cert path as param** — simpler, avoids async refactor of verify_full pipeline
- Startup in lib.rs already does `block_on` for DB reads, so reading cert path there is trivial
- `import_license_file` already has DB access + does `block_on`-equivalent via async context
- `extract_spki_der` in callbacks.rs works on DER only — comm cert may be PEM. Use `cert_parser` module's PEM detection + x509_parser directly.

## Requirements
### Functional
- `is_licensed(pkcs11_path, app_data_dir, comm_cert_path: Option<&str>)` — new 3rd param
- `verify_full()` same signature change
- If `comm_cert_path` is None/empty -> return `LicenseError::NoCommunicationCert`
- Read cert file -> parse X.509 -> extract SPKI DER -> `RsaPublicKey::from_public_key_der`
- Pass `RsaPublicKey` to `verify_license_signature(payload, sig, &public_key)`
- Remove `#[cfg(debug_assertions)]` skip logic for signature verification (lines 65-68 in mod.rs)
- Update lib.rs startup to read `communication_cert_path` from DB settings
- Update `import_license_file` to read cert path and pass to `is_licensed`

### Non-functional
- No new crate dependencies (x509-parser + rsa already available)
- Sync function preserved (no async conversion)

## Architecture

### Data Flow
```
lib.rs startup:
  block_on(get_setting(pool, "communication_cert_path"))
  -> is_licensed(pkcs11_path, app_data_dir, comm_cert_path.as_deref())

is_licensed -> verify_full:
  comm_cert_path = Some(path) or return NoCommunicationCert
  bytes = fs::read(path)
  (PEM check) -> x509_parser::parse_x509_certificate(der)
  spki_der = cert.public_key().raw.to_vec()
  public_key = RsaPublicKey::from_public_key_der(&spki_der)
  verify_license_signature(payload, sig, &public_key)
```

### Failure Modes
| Failure | Error | User Impact |
|---------|-------|-------------|
| No comm cert in DB | NoCommunicationCert | UI: "Import server cert in Settings" |
| Cert file deleted from disk | NoCommunicationCert | Same prompt |
| Cert not RSA | InvalidKey | "License file invalid" |
| Cert RSA key != signing key | Corrupted | "Tampered with" |
| Cert expired | Still works (key extraction doesn't check validity) | None |

## Related Code Files
- **Modify:** `src-tauri/src/license/mod.rs`
- **Modify:** `src-tauri/src/lib.rs` (startup, lines 114-131)
- **Modify:** `src-tauri/src/commands/license.rs` (import_license_file, line 226)

## Implementation Steps

### A. Update mod.rs

1. Add imports at top:
   ```rust
   use rsa::{pkcs8::DecodePublicKey, RsaPublicKey};
   use x509_parser::prelude::*;
   ```

2. Change `is_licensed` signature (line 16):
   ```rust
   pub fn is_licensed(pkcs11_lib_path: &str, app_data_dir: &Path, comm_cert_path: Option<&str>) -> LicenseInfo {
       match verify_full(pkcs11_lib_path, app_data_dir, comm_cert_path) {
   ```

3. Change `verify_full` signature (line 28):
   ```rust
   fn verify_full(pkcs11_lib_path: &str, app_data_dir: &Path, comm_cert_path: Option<&str>) -> Result<LicenseInfo, LicenseError> {
   ```

4. Add cert extraction after Phase A (after line 58, before signature verification). Replace the `#[cfg]` block (lines 65-68):
   ```rust
   // Extract public key from communication certificate
   let comm_path = comm_cert_path
       .filter(|p| !p.is_empty())
       .ok_or(LicenseError::NoCommunicationCert)?;
   
   if !std::path::Path::new(comm_path).exists() {
       return Err(LicenseError::NoCommunicationCert);
   }
   
   let cert_data = std::fs::read(comm_path)
       .map_err(|e| LicenseError::InvalidKey(format!("Cannot read communication cert: {}", e)))?;
   
   let public_key = extract_public_key_from_cert(&cert_data)?;
   
   payload::verify_license_signature(&payload_bytes, &sig_bytes, &public_key)?;
   ```

5. Remove the old `#[cfg(not(debug_assertions))]` / `#[cfg(debug_assertions)]` lines 65-68 entirely.

6. Add helper function at bottom of mod.rs:
   ```rust
   /// Extract RSA public key from X.509 certificate bytes (auto-detects PEM/DER).
   fn extract_public_key_from_cert(cert_data: &[u8]) -> Result<RsaPublicKey, LicenseError> {
       // Detect PEM and convert to DER if needed
       let der_bytes: Vec<u8> = if cert_data.windows(b"-----BEGIN".len()).any(|w| w == b"-----BEGIN") {
           let (_, pem) = x509_parser::pem::parse_x509_pem(cert_data)
               .map_err(|e| LicenseError::InvalidKey(format!("PEM parse error: {:?}", e)))?;
           pem.contents.clone()
       } else {
           cert_data.to_vec()
       };
   
       let (_, cert) = parse_x509_certificate(&der_bytes)
           .map_err(|e| LicenseError::InvalidKey(format!("Certificate parse error: {:?}", e)))?;
   
       let spki_der = cert.public_key().raw.to_vec();
       RsaPublicKey::from_public_key_der(&spki_der)
           .map_err(|e| LicenseError::InvalidKey(format!("Not an RSA certificate: {}", e)))
   }
   ```

### B. Update lib.rs startup (lines 114-131)

7. Inside the `license_info` block, read comm cert path from DB:
   ```rust
   let license_info = {
       // ... existing pkcs11_path resolution stays ...
       
       let comm_cert_path = tauri::async_runtime::block_on(async {
           crate::db::settings_repo::get_setting(&pool, "communication_cert_path").await.ok().flatten()
       });
       
       license::is_licensed(&pkcs11_path, &app_data_dir, comm_cert_path.as_deref())
   };
   ```

### C. Update commands/license.rs (import_license_file, line 226)

8. Before calling `license::is_licensed`, read comm cert path:
   ```rust
   let comm_cert_path = crate::db::settings_repo::get_setting(&state.db, "communication_cert_path")
       .await
       .map_err(|e| e.to_string())?;
   
   let new_info = license::is_licensed(&pkcs11_path, &app_data_dir, comm_cert_path.as_deref());
   ```

## Todo List
- [x] Add imports to mod.rs (rsa, x509_parser)
- [x] Update `is_licensed` signature — add comm_cert_path param
- [x] Update `verify_full` signature — add comm_cert_path param
- [x] Add cert extraction + public key parsing in verify_full
- [x] Remove `#[cfg(debug_assertions)]` signature skip logic
- [x] Add `extract_public_key_from_cert` helper function
- [x] Update lib.rs startup to read comm_cert_path from DB
- [x] Update import_license_file to pass comm_cert_path

## Success Criteria
- `cargo check` passes (debug AND release)
- No `compile_error!` blocks remain
- No `#[cfg(debug_assertions)]` skip logic for signatures
- `is_licensed` requires 3 params at all call sites
- Missing comm cert -> `NoCommunicationCert` status returned

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Existing debug workflow breaks (no comm cert in dev) | High | Medium | Dev must import comm cert before license activation — acceptable per user confirmation |
| cert_parser PEM detection differs from extract_public_key_from_cert | Low | Low | Both use same `-----BEGIN` check pattern |
| x509_parser version incompatibility | None | N/A | Already using 0.16 throughout codebase |

## Security Considerations
- **Key trust:** Public key now comes from user-imported cert, not hardcoded. Trust anchored in IT admin importing correct server cert.
- **No PIN needed:** Public key extraction is a parse operation, no crypto hardware involved.
- **Cert file integrity:** File read from app_data_dir (copied by save_communication_cert). Filesystem-level protection only. Acceptable for client-side license check.
- **Signature verification mandatory:** Removing debug skip means dev builds also verify signatures (need valid comm cert + license.dat pair for testing).

## Next Steps
- Phase 4: Compile verification
- Frontend: Handle `no_communication_cert` status in LicenseGate component (separate task)
