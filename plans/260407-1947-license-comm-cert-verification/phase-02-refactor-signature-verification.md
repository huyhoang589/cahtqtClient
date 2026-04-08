# Phase 2: Refactor Signature Verification

## Context Links
- [payload.rs](../../src-tauri/src/license/payload.rs) — verify_license_signature(), SERVER_PUBLIC_KEY_PEM
- [callbacks.rs](../../src-tauri/src/htqt_ffi/callbacks.rs) — extract_spki_der() pattern (line 228)
- [plan.md](./plan.md)

## Overview
- **Priority:** P1 (blocks Phase 3)
- **Status:** Complete
- **Description:** Remove hardcoded `SERVER_PUBLIC_KEY_PEM` + `compile_error!`, change `verify_license_signature()` to accept `&RsaPublicKey` parameter instead of reading from constant.

## Key Insights
- `RsaPublicKey::from_public_key_der()` is the right import path (already used in callbacks.rs)
- `x509_parser` cert `.public_key().raw` returns SPKI DER bytes — these work directly with `from_public_key_der()`
- Caller (mod.rs Phase 3) will extract key and pass it in
- `validate_license_file_structure()` only does structural checks, does NOT need public key

## Requirements
### Functional
- Remove `SERVER_PUBLIC_KEY_PEM` constant (lines 20-26)
- Remove `compile_error!` guard (lines 17-18)
- `verify_license_signature(payload, sig, public_key)` takes `&RsaPublicKey` as 3rd param
- Remove unused `DecodePublicKey` import

### Non-functional
- `validate_license_file_structure()` unchanged (structural only)
- No new dependencies needed

## Architecture
```
Before: verify_license_signature(payload, sig) -> reads SERVER_PUBLIC_KEY_PEM internally
After:  verify_license_signature(payload, sig, &RsaPublicKey) -> uses caller-provided key
```

## Related Code Files
- **Modify:** `src-tauri/src/license/payload.rs`

## Implementation Steps

1. Open `src-tauri/src/license/payload.rs`
2. **Remove** lines 14-26 (SERVER_PUBLIC_KEY_PEM constant + compile_error! + doc comment):
   ```rust
   // DELETE: lines 14-26 entirely
   ```
3. **Remove** `DecodePublicKey` from import on line 5:
   ```rust
   // Before:
   use rsa::{pkcs8::DecodePublicKey, RsaPublicKey};
   // After:
   use rsa::RsaPublicKey;
   ```
4. **Change** `verify_license_signature` signature (line 81) to accept public key:
   ```rust
   pub fn verify_license_signature(payload: &[u8], sig: &[u8], public_key: &RsaPublicKey) -> Result<(), LicenseError> {
   ```
5. **Remove** the internal key parsing (line 82-83), use parameter directly:
   ```rust
   // Before:
   let public_key = RsaPublicKey::from_public_key_pem(SERVER_PUBLIC_KEY_PEM)
       .map_err(|e| LicenseError::InvalidKey(format!("Server public key invalid: {}", e)))?;
   // After: (delete these lines, public_key comes from parameter)
   ```
6. Update `verifying_key` line to use reference:
   ```rust
   let verifying_key = VerifyingKey::<Sha256>::new(public_key.clone());
   ```

## Todo List
- [x] Remove SERVER_PUBLIC_KEY_PEM constant + compile_error!
- [x] Remove DecodePublicKey import
- [x] Add `public_key: &RsaPublicKey` parameter to `verify_license_signature`
- [x] Remove internal key parsing, use parameter
- [x] Verify `validate_license_file_structure` needs no changes

## Success Criteria
- `cargo check` passes (will fail until Phase 3 updates callers)
- No references to SERVER_PUBLIC_KEY_PEM remain
- `verify_license_signature` requires 3 args

## Risk Assessment
- **Medium:** Callers break until Phase 3 updates them — expected, sequential phases
- **Mitigation:** Phases 2+3 should be implemented in same commit

## Security Considerations
- Removes placeholder key — prevents accidental release with dummy key
- Signature verification now mandatory (no bypass path)

## Next Steps
- Phase 3 updates all callers to provide `RsaPublicKey` from comm cert
