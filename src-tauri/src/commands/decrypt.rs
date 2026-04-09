use std::ffi::c_void;
use std::path::Path;
use std::ptr;
use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::{
    app_log::emit_app_log,
    db::{logs_repo, settings_repo},
    etoken::models::TokenStatus,
    htqt_ffi::{
        callbacks,
        error_codes::HTQT_BATCH_OVERWRITE_OUTPUT,
        htqt_error_display,
        token_context::open_token_session, CryptoCallbacksV2,
    },
    AppState,
};

#[derive(Serialize, Clone)]
pub struct DecryptProgress {
    pub current: usize,
    pub total: usize,
    pub file_name: String,
    pub file_path: String,
    pub status: String, // "processing" | "success" | "error"
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct DecryptResult {
    pub total: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

/// Decrypt M .sf1 files using decrypt_one_sfv1 with callback-based crypto.
/// Recipient matched via own_cert_der fingerprint — no recipient_id needed.
#[tauri::command]
pub async fn decrypt_batch(
    app: AppHandle,
    file_paths: Vec<String>,
    output_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<DecryptResult, String> {
    // Atomically set running=true; fail if already true (TOCTOU-safe)
    if state.is_operation_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Another operation is already running".to_string());
    }
    let result = run_decrypt_batch(&app, &file_paths, output_dir.as_deref(), &state).await;
    state.is_operation_running.store(false, Ordering::SeqCst);
    result
}

async fn run_decrypt_batch(
    app: &AppHandle,
    file_paths: &[String],
    output_dir_override: Option<&str>,
    state: &State<'_, AppState>,
) -> Result<DecryptResult, String> {
    // Read and validate token login state
    let (pkcs11_lib, slot_id, pin_str) = {
        let login = state.token_login.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        if login.status != TokenStatus::LoggedIn {
            return Err("Token not logged in — login via Settings first".to_string());
        }
        let pin = login.get_pin().ok_or("PIN not available — re-login required")?.to_string();
        (
            login.pkcs11_lib_path.clone().unwrap_or_default(),
            login.slot_id.unwrap_or(0),
            pin,
        )
    };

    // Get caller's own cert DER (required for SF v1 fingerprint matching)
    let own_cert_der: Vec<u8> = {
        let scan = state.last_token_scan.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        scan.as_ref()
            .and_then(|s| s.certificates.first())
            .map(|e| e.certificate.raw_der.clone())
            .unwrap_or_default()
    };

    // Resolve output directory: use override if provided, else output_data_dir\SF\DECRYPT
    let output_dir_str = if let Some(dir) = output_dir_override {
        dir.to_string()
    } else {
        let base = settings_repo::get_setting(&state.db, "output_data_dir")
            .await
            .ok()
            .flatten()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| {
                std::env::var("USERPROFILE")
                    .map(|p| format!("{}\\Desktop", p))
                    .unwrap_or_default()
            });
        format!("{}\\SF\\DECRYPT", base.trim_end_matches(['/', '\\']))
    };

    std::fs::create_dir_all(&output_dir_str)
        .map_err(|e| format!("Cannot create output directory: {}", e))?;

    let total = file_paths.len();
    emit_app_log(app, "info", &format!("Starting decryption: {} file(s)", total));

    // Open PKCS#11 session once for all files (reuse session + callbacks)
    let htqt_lib_arc = state.htqt_lib.clone();
    let app_clone = app.clone();
    let own_cert_der_clone = own_cert_der.clone();
    let file_paths_owned = file_paths.to_vec();
    let output_dir_str_clone = output_dir_str.clone();

    // Run all decryption synchronously in spawn_blocking
    // Each result: (input_path, output_path_from_dll, Result<Ok, (code, detail)>)
    let dec_results = tokio::task::spawn_blocking(move || -> Result<Vec<(String, Result<String, (i32, String)>)>, String> {
        let ctx = open_token_session(
            &pkcs11_lib,
            slot_id,
            &pin_str,
            app_clone,
            own_cert_der_clone.clone(),
            "decrypt-progress".to_string(),
        )?;

        let ctx_box = Box::new(ctx);
        let user_ctx_ptr = &*ctx_box as *const _ as *mut c_void;

        let cbs = CryptoCallbacksV2 {
            sign_fn: None, // not needed for decrypt
            rsa_dec_fn: Some(callbacks::cb_rsa_oaep_decrypt),
            progress_fn: None, // decrypt does not use progress callback
            user_ctx: user_ctx_ptr,
            own_cert_der: if own_cert_der_clone.is_empty() { ptr::null() } else { own_cert_der_clone.as_ptr() },
            own_cert_der_len: own_cert_der_clone.len() as u32,
            reserved: [ptr::null_mut(); 3],
        };

        let guard = htqt_lib_arc.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        let lib = guard.as_ref().ok_or("htqt_crypto.dll not loaded")?;

        let mut results = Vec::with_capacity(file_paths_owned.len());
        for file_path in &file_paths_owned {
            let dec_result = lib.decrypt_one_sfv1(file_path, &output_dir_str_clone, &cbs, HTQT_BATCH_OVERWRITE_OUTPUT);
            results.push((file_path.clone(), dec_result));
        }

        drop(guard);
        drop(ctx_box); // closes session + finalizes Pkcs11

        Ok(results)
    })
    .await
    .map_err(|e| e.to_string())??;

    // Emit progress events and log to DB
    let mut success_count = 0usize;
    let mut error_count = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for (i, (file_path, result)) in dec_results.iter().enumerate() {
        let current = i + 1;
        let file_name = Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.clone());

        let (status_str, error_msg, out_path) = match result {
            Ok(output_path) => {
                success_count += 1;
                ("success".to_string(), None, output_path.as_str())
            }
            Err((code, detail)) => {
                error_count += 1;
                let base = htqt_error_display(*code);
                let error_str = if detail.is_empty() {
                    base
                } else {
                    format!("{} — {}", base, detail)
                };
                errors.push(format!("{}: {}", file_name, error_str));
                ("error".to_string(), Some(error_str), "")
            }
        };

        // Emit per-file progress event
        let _ = app.emit("decrypt-progress", DecryptProgress {
            current,
            total,
            file_name: file_name.clone(),
            file_path: file_path.clone(),
            status: status_str.clone(),
            error: error_msg.clone(),
        });

        // Log to database
        let _ = logs_repo::insert_log(
            &state.db,
            "DECRYPT",
            file_path,
            out_path,
            None,
            &status_str,
            error_msg.as_deref(),
        )
        .await;
    }

    emit_app_log(
        app,
        if error_count == 0 { "success" } else { "warning" },
        &format!("Decryption: {}/{} succeeded", success_count, total),
    );
    Ok(DecryptResult { total, success_count, error_count, errors })
}
