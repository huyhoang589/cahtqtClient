## Phase 1: Update FFI Types

### Overview
- **Priority**: High
- **Status**: complete
- **Description**: Update `types.rs` to match new `htqt-api.h` — remove deprecated callback types, update CryptoCallbacksV2 struct, update DLL function pointer types.

### Related Code Files
- **Modify**: `src-tauri/src/htqt_ffi/types.rs`
- **Reference**: `feature/3. cryptoApi/htqt-api.h`

### Implementation Steps

1. **Remove deprecated callback typedefs**
   - Delete `FnRsaOaepEncryptForCert` (lines ~14-23)
   - Delete `FnRsaPssVerify` (lines ~34-43)

2. **Update `CryptoCallbacksV2` struct** (lines ~78-95) to match new header:
   ```rust
   #[repr(C)]
   pub struct CryptoCallbacksV2 {
       pub sign_fn: Option<FnRsaPssSign>,
       pub rsa_dec_fn: Option<FnRsaOaepDecrypt>,
       pub progress_fn: Option<FnProgressCallback>,
       pub user_ctx: *mut c_void,
       pub own_cert_der: *const u8,
       pub own_cert_der_len: c_uint,
       pub reserved: [*mut c_void; 3],
   }
   ```
   Key: removed `rsa_enc_cert_fn` and `verify_fn` fields.

3. **Update `FnEncHTQTMulti`** (lines ~57-63) to match `encHTQT_sf_multi`:
   ```rust
   pub type FnEncHTQTMulti = unsafe extern "C" fn(
       params: *const BatchEncryptParams,
       cbs: *const CryptoCallbacksV2,
       results: *mut BatchResult,
       error_msg: *mut c_char,
       error_len: c_int,
   ) -> c_int;
   ```

4. **Update `FnDecHTQTV2`** (lines ~65-72) to match `decrypt_one_sfv1`:
   ```rust
   pub type FnDecHTQTV2 = unsafe extern "C" fn(
       sf1_path: *const c_char,
       output_dir: *const c_char,
       cbs: *const CryptoCallbacksV2,
       flags: u32,
       out_path_buf: *mut c_char,
       out_path_buf_len: c_int,
       err_buf: *mut c_char,
       err_len: c_int,
   ) -> c_int;
   ```

5. **Verify BatchEncryptParams, BatchResult, FileEntry, RecipientEntry** match new header:
   - `BatchEncryptParams` now has `flags: u32` and `reserved: [*mut c_void; 2]` — verify these exist
   - `BatchResult` unchanged structurally

### Todo
- [x] Remove FnRsaOaepEncryptForCert typedef
- [x] Remove FnRsaPssVerify typedef
- [x] Update CryptoCallbacksV2 (remove 2 fields)
- [x] Update FnEncHTQTMulti signature
- [x] Update FnDecHTQTV2 signature
- [x] Verify batch structs match new header

### Success Criteria
- `types.rs` compiles (may have unused import warnings until callbacks.rs is updated)
- Struct layout matches `htqt-api.h` exactly (field order matters for FFI)
