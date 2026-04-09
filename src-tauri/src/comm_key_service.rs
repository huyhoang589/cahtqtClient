//! Centralized communication key (.sf1) service module.
//! Provides decrypt, cleanup, and lookup functions used by SET KEY, LICENSE, and ENCRYPT flows.

use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::ptr;

use tauri::AppHandle;

use crate::htqt_ffi::{callbacks, token_context::open_token_session, CryptoCallbacksV2};

/// Decrypt .sf1 communication key → returns temp cert path.
/// Requires active PKCS#11 session (token must be logged in).
/// Runs synchronously — caller wraps in spawn_blocking if needed.
pub fn decrypt_comm_key(
    sf1_path: &str,
    temp_dir: &str,
    htqt_lib: &crate::htqt_ffi::HtqtLib,
    pkcs11_lib: &str,
    slot_id: u32,
    pin: &str,
    own_cert_der: Vec<u8>,
    app: AppHandle,
) -> Result<String, String> {
    // Open a dedicated PKCS#11 session for decryption
    let ctx = open_token_session(
        pkcs11_lib,
        slot_id,
        pin,
        app,
        own_cert_der.clone(),
        "comm-key-decrypt".to_string(),
    )?;

    let ctx_box = Box::new(ctx);
    let user_ctx_ptr = &*ctx_box as *const _ as *mut c_void;

    let cbs = CryptoCallbacksV2 {
        sign_fn: None,
        rsa_dec_fn: Some(callbacks::cb_rsa_oaep_decrypt),
        progress_fn: None,
        user_ctx: user_ctx_ptr,
        own_cert_der: if own_cert_der.is_empty() { ptr::null() } else { own_cert_der.as_ptr() },
        own_cert_der_len: own_cert_der.len() as u32,
        reserved: [ptr::null_mut(); 3],
    };

    let output_path = htqt_lib
        .decrypt_one_sfv1(sf1_path, temp_dir, &cbs, 0)
        .map_err(|(code, detail)| {
            format!("Failed to decrypt communication key (code {}): {}", code, detail)
        })?;

    drop(ctx_box); // closes PKCS#11 session

    Ok(output_path)
}

/// Delete a temp cert file (best-effort, logs on error).
pub fn cleanup_temp_cert(cert_path: &str) {
    if cert_path.is_empty() {
        return;
    }
    if let Err(e) = std::fs::remove_file(cert_path) {
        eprintln!("[comm_key_service] cleanup_temp_cert failed for {}: {}", cert_path, e);
    }
}

/// Startup: delete orphaned decrypted cert files in temp dir.
/// Deletes .crt/.cer/.pem/.der files NOT referenced by any PartnerMember cert_file_path.
pub fn cleanup_orphaned_certs(temp_dir: &Path, referenced_paths: &[String]) {
    let entries = match std::fs::read_dir(temp_dir) {
        Ok(e) => e,
        Err(_) => return, // dir doesn't exist or not readable — no-op
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !["crt", "cer", "pem", "der"].contains(&ext.as_str()) {
            continue;
        }
        let path_str = path.to_string_lossy().to_string();
        // Normalize slashes for comparison
        let path_normalized = path_str.replace('\\', "/");
        let is_referenced = referenced_paths.iter().any(|rp| {
            rp.replace('\\', "/") == path_normalized
        });
        if !is_referenced {
            if let Err(e) = std::fs::remove_file(&path) {
                eprintln!("[comm_key_service] orphan cleanup failed for {:?}: {}", path, e);
            }
        }
    }
}

/// Get path to stored .sf1 file in COMM_KEY dir (if exists).
pub fn get_stored_comm_key_path(comm_key_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(comm_key_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext.eq_ignore_ascii_case("sf1") {
                return Some(path);
            }
        }
    }
    None
}

/// RAII guard that ensures temp cert is deleted on drop (even on panic).
pub struct TempCertGuard {
    pub path: Option<String>,
}

impl Drop for TempCertGuard {
    fn drop(&mut self) {
        if let Some(ref p) = self.path {
            cleanup_temp_cert(p);
        }
    }
}
