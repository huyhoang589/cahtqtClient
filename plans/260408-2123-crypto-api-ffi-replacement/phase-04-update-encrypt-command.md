## Phase 4: Update Encrypt Command

### Overview
- **Priority**: High
- **Status**: complete
- **Description**: Update `encrypt.rs` for new API: fewer callbacks in CryptoCallbacksV2, results capacity = file_count, output .sf1 naming.

### Related Code Files
- **Modify**: `src-tauri/src/commands/encrypt.rs`
- **Depends on**: Phases 1-3

### Implementation Steps

1. **Update CryptoCallbacksV2 initialization** (in spawn_blocking block, ~lines 230-260):
   ```rust
   let cbs = CryptoCallbacksV2 {
       sign_fn: Some(cb_rsa_pss_sign),
       rsa_dec_fn: None,           // not needed for encrypt
       progress_fn: Some(cb_progress),
       user_ctx: ctx_ptr as *mut c_void,
       own_cert_der: own_cert_ptr,
       own_cert_der_len: own_cert_len as c_uint,
       reserved: [std::ptr::null_mut(); 3],
   };
   ```
   Remove: `rsa_enc_cert_fn` and `verify_fn` fields (no longer in struct).

2. **Update results buffer capacity** (~line 220):
   - Old: `vec![BatchResult::zeroed(); file_count * recipient_count]`
   - New: `vec![BatchResult::zeroed(); file_count]`
   - SF v1 produces one output per file (all recipients embedded in single .sf1)

3. **Update result processing loop** (~lines 306-384):
   - Old: iterates `file_count * recipient_count` results
   - New: iterates `file_count` results
   - Each result corresponds to one input file → one output .sf1
   - Progress total = file_count (not M×N)

4. **Update output file naming expectations**:
   - Old: `{file_id}-{recipient_id}.sf`
   - New: `{file_id}.sf1`
   - Update any path construction or display logic

5. **Update BatchEncryptParams** if struct changed:
   - Verify `flags` field is set (0 or HTQT_BATCH_CONTINUE_ON_ERROR etc.)
   - Verify `reserved` fields are null

6. **Update progress event emission**:
   - Progress callback still fires per (file_idx, recip_idx) pair during encryption
   - But final result count = file_count
   - Frontend progress tracking may need adjustment

### Todo
- [x] Update CryptoCallbacksV2 init (remove 2 fields)
- [x] Change results capacity to file_count
- [x] Update result processing loop (file_count iterations)
- [x] Verify BatchEncryptParams flags/reserved fields
- [x] Update progress total calculation
- [x] Test output naming (.sf1)

### Success Criteria
- Encrypt command compiles with new types
- Results buffer sized correctly (file_count)
- CryptoCallbacksV2 matches new struct layout
