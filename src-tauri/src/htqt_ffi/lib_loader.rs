use std::ffi::CString;

use libloading::{os::windows::Library as WinLib, Library};

use super::types::*;
use super::DLL_LOCK;

/// htqt_crypto DLL wrapper — resolves 3 symbols: encHTQT_sf_multi, decrypt_one_sfv1, HTQT_GetError.
pub struct HtqtLib {
    #[allow(dead_code)]
    lib: Library, // kept alive so typed fn pointers remain valid
    enc_multi_fn: FnEncHTQTMulti,
    dec_sfv1_fn: FnDecHTQTV2,
    #[allow(dead_code)]
    get_error_fn: FnGetError,
}

// Accessed through Arc<Mutex<Option<HtqtLib>>> in AppState — safe to mark.
unsafe impl Send for HtqtLib {}
unsafe impl Sync for HtqtLib {}

impl HtqtLib {
    /// Load htqt_crypto.dll from path and resolve v2 symbols.
    /// Uses LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR (0x100) so the DLL's own dependencies
    /// are found in the same directory, plus LOAD_LIBRARY_SEARCH_DEFAULT_DIRS (0x1000)
    /// for system DLLs.
    pub fn load(path: &str) -> Result<Self, String> {
        const LOAD_FLAGS: u32 = 0x0000_0100 | 0x0000_1000;
        let lib: Library = unsafe {
            WinLib::load_with_flags(path, LOAD_FLAGS)
                .map(Library::from)
                .map_err(|e| format!("Failed to load htqt_crypto.dll: {}", e))?
        };

        let enc_multi_fn: FnEncHTQTMulti = unsafe {
            *lib.get::<FnEncHTQTMulti>(b"encHTQT_sf_multi\0")
                .map_err(|_| "Symbol 'encHTQT_sf_multi' not found in htqt_crypto.dll".to_string())?
        };

        let dec_sfv1_fn: FnDecHTQTV2 = unsafe {
            *lib.get::<FnDecHTQTV2>(b"decrypt_one_sfv1\0")
                .map_err(|_| "Symbol 'decrypt_one_sfv1' not found in htqt_crypto.dll".to_string())?
        };

        let get_error_fn: FnGetError = unsafe {
            *lib.get::<FnGetError>(b"HTQT_GetError\0")
                .map_err(|_| "Symbol 'HTQT_GetError' not found in htqt_crypto.dll".to_string())?
        };

        Ok(HtqtLib { lib, enc_multi_fn, dec_sfv1_fn, get_error_fn })
    }

    /// Batch encrypt M files × N recipients via encHTQT_sf_multi.
    /// results slice must have capacity >= file_count (one .sf1 per input file).
    /// Returns Ok(rc): 0 = all success, >0 = partial failures in results.
    pub fn enc_multi(
        &self,
        params: &BatchEncryptParams,
        cbs: &CryptoCallbacksV2,
        results: &mut [BatchResult],
    ) -> Result<i32, String> {
        let mut err_buf = [0i8; 512];
        let _guard = DLL_LOCK.lock().map_err(|_| "DLL_LOCK poisoned".to_string())?;

        let rc = unsafe {
            (self.enc_multi_fn)(params, cbs, results.as_mut_ptr(), err_buf.as_mut_ptr(), 512)
        };

        if rc < 0 {
            let msg = unsafe { std::ffi::CStr::from_ptr(err_buf.as_ptr()) }
                .to_string_lossy()
                .to_string();
            Err(format!("encHTQT_multi failed ({}): {}", rc, msg))
        } else {
            Ok(rc)
        }
    }

    /// Decrypt a single SF v1 (.sf1) file via decrypt_one_sfv1.
    /// On success returns Ok(output_path). On failure returns Err((rc, detail)).
    pub fn decrypt_one_sfv1(
        &self,
        sf1_path: &str,
        output_dir: &str,
        cbs: &CryptoCallbacksV2,
        flags: u32,
    ) -> Result<String, (i32, String)> {
        let sf = CString::new(sf1_path).map_err(|e| (-1, e.to_string()))?;
        let out = CString::new(output_dir).map_err(|e| (-1, e.to_string()))?;
        let mut out_path_buf = [0i8; 512];
        let mut err_buf = [0i8; 512];

        let _guard = DLL_LOCK.lock().map_err(|_| (-1, "DLL_LOCK poisoned".to_string()))?;

        let rc = unsafe {
            (self.dec_sfv1_fn)(
                sf.as_ptr(),
                out.as_ptr(),
                cbs,
                flags,
                out_path_buf.as_mut_ptr(),
                512,
                err_buf.as_mut_ptr(),
                512,
            )
        };

        if rc != 0 {
            let detail = unsafe { std::ffi::CStr::from_ptr(err_buf.as_ptr()) }
                .to_string_lossy()
                .to_string();
            Err((rc, detail))
        } else {
            let out_path = unsafe { std::ffi::CStr::from_ptr(out_path_buf.as_ptr()) }
                .to_string_lossy()
                .to_string();
            Ok(out_path)
        }
    }
}
