## Phase 2: Update DLL Loader

### Overview
- **Priority**: High
- **Status**: complete
- **Description**: Update `lib_loader.rs` to load new symbol names and update wrapper method signatures.

### Related Code Files
- **Modify**: `src-tauri/src/htqt_ffi/lib_loader.rs`
- **Depends on**: Phase 1 (updated type definitions)

### Implementation Steps

1. **Update symbol names** in `load()` (lines ~35-48):
   - `encHTQT_multi` → `encHTQT_sf_multi`
   - `decHTQT_v2` → `decrypt_one_sfv1`
   - `HTQT_GetError` — unchanged

2. **Update `enc_multi()` wrapper** (lines ~53-77):
   - Signature matches new `FnEncHTQTMulti` — takes `(params, cbs, results, error_msg, error_len)`
   - Should already be compatible if types.rs is updated correctly
   - Verify DLL_LOCK usage unchanged

3. **Update `dec_v2()` wrapper** (lines ~79-107):
   - New signature: `(sf1_path, output_dir, cbs, flags, out_path_buf, out_path_buf_len, err_buf, err_len)`
   - **Remove** `recipient_id` parameter
   - **Add** `flags: u32` parameter
   - **Add** `out_path_buf` + `out_path_buf_len` for output path return
   - Rename method to `decrypt_one_sfv1()` (confirmed in validation)
   <!-- Updated: Validation Session 1 - Rename dec_v2() → decrypt_one_sfv1() confirmed -->
   - Update error extraction: new API returns error in `err_buf` directly

### Todo
- [x] Change DLL symbol name for encrypt
- [x] Change DLL symbol name for decrypt
- [x] Update dec_v2() signature (remove recipient_id, add flags + out_path_buf + err_buf)
- [x] Update enc_multi() if needed
- [x] Verify error handling paths

### Success Criteria
- DLL symbols resolve correctly at runtime with new DLL
- Wrapper methods expose correct signatures to callers
