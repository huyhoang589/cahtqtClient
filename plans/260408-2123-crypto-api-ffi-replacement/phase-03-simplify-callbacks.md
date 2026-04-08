## Phase 3: Simplify Callbacks

### Overview
- **Priority**: High
- **Status**: complete
- **Description**: Remove callbacks now handled internally by DLL (OAEP encrypt, PSS verify). Keep sign, decrypt, progress.

### Related Code Files
- **Modify**: `src-tauri/src/htqt_ffi/callbacks.rs`
- **Depends on**: Phase 1 (removed type definitions)

### Implementation Steps

1. **Delete `cb_rsa_oaep_enc_cert()`** (lines ~79-120)
   - DLL now handles RSA-OAEP encryption internally using recipient cert public keys

2. **Delete `cb_rsa_pss_verify()`** (lines ~165-203)
   - DLL now handles RSA-PSS signature verification internally

3. **Delete `extract_spki_der()`** helper (lines ~228-232)
   - Only used by deleted callbacks
   - If `cb_rsa_pss_verify` or `cb_rsa_oaep_enc_cert` were the only callers, safe to remove

4. **Remove unused imports** after deletions:
   - `rsa` crate imports (OaepEncryption, pkcs1v15, etc.) — only if no remaining callers
   - `x509_parser` — only if extract_spki_der was the only user
   - Check if `cb_rsa_pss_sign` or `cb_rsa_oaep_decrypt` still need any of these

5. **Keep intact**:
   - `cb_rsa_pss_sign()` (lines ~35-75) — still needed for token signing
   - `cb_rsa_oaep_decrypt()` (lines ~124-161) — still needed for token decryption
   - `cb_progress()` (lines ~207-223) — still needed for progress events

### Todo
- [x] Delete cb_rsa_oaep_enc_cert()
- [x] Delete cb_rsa_pss_verify()
- [x] Delete extract_spki_der()
- [x] Clean up unused imports
- [x] Verify remaining callbacks compile

### Success Criteria
- callbacks.rs contains only 3 callbacks: sign, decrypt, progress
- No unused imports or dead code warnings
- File reduced from ~233 lines to ~130 lines
