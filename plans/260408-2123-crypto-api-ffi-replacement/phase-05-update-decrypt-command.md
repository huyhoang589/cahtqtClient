## Phase 5: Update Decrypt Command

### Overview
- **Priority**: High
- **Status**: complete
- **Description**: Update `decrypt.rs` for new `decrypt_one_sfv1` API: no recipient_id, cert fingerprint matching, new signature with flags + output path buffer.

### Related Code Files
- **Modify**: `src-tauri/src/commands/decrypt.rs`
- **Depends on**: Phases 1-3

### Implementation Steps

1. **Update CryptoCallbacksV2 initialization** (~lines 140-155):
   ```rust
   let cbs = CryptoCallbacksV2 {
       sign_fn: None,              // not needed for decrypt
       rsa_dec_fn: Some(cb_rsa_oaep_decrypt),
       progress_fn: Some(cb_progress),
       user_ctx: ctx_ptr as *mut c_void,
       own_cert_der: own_cert_ptr,
       own_cert_der_len: own_cert_len as c_uint,
       reserved: [std::ptr::null_mut(); 3],
   };
   ```
   Remove: `rsa_enc_cert_fn` and `verify_fn` fields.

2. **Update DLL call** (~lines 155-170):
   - Old: `lib.dec_v2(sf_path, dst_path, recipient_id, &cbs)`
   - New: `lib.decrypt_one_sfv1(sf1_path, output_dir, &cbs, flags, out_path_buf, out_path_buf_len, err_buf, err_len)` (or however renamed in phase 2)
   - **Remove** `recipient_id` parameter — DLL matches recipient via `own_cert_der` fingerprint
   - **Add** `flags: u32` (0 or HTQT_BATCH_OVERWRITE_OUTPUT)
   - **Add** output path buffer for DLL to write decrypted file path
   - **Add** error buffer for DLL to write error detail

3. **Remove recipient_id and partnerName** (~lines 68-82):
   - Old: reads `cert_cn` from token_login as recipient_id
   - New: not needed — DLL uses `own_cert_der` for matching
   - Keep reading own_cert_der from last_token_scan (still required)
   - Remove `partner_name` from command args — no longer used for output dir naming (DLL writes path via out_path_buf)
   <!-- Updated: Validation Session 1 - Remove partnerName from decrypt command -->

4. **Update output path handling**:
   - Old: we construct output path and pass to DLL
   - New: DLL writes output path into `out_path_buf` — read it back after call
   - Still pass `output_dir` for DLL to know where to write

5. **Update error handling**:
   - New API returns error detail in `err_buf` directly
   - Update error extraction from DLL response

6. **Update per-file result processing** (~lines 180-238):
   - Use output path from DLL's `out_path_buf` instead of constructing it
   - Progress events: file_name from DLL output or original input path

### Todo
- [x] Update CryptoCallbacksV2 init (remove 2 fields)
- [x] Update DLL call to new signature (no recipient_id, add flags + buffers)
- [x] Remove recipient_id extraction logic
- [x] Handle out_path_buf from DLL
- [x] Update error handling for err_buf
- [x] Update progress event emission

### Success Criteria
- Decrypt command compiles with new types
- DLL called with correct new signature
- Output path correctly read from DLL response
- No recipient_id dependency
