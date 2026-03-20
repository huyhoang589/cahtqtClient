use std::ffi::{CString, OsString, c_void};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::Path;
use std::ptr;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use tauri::Emitter;

use crate::{
    app_log::emit_app_log,
    cert_parser,
    db::{logs_repo, settings_repo},
    etoken::models::TokenStatus,
    htqt_ffi::{
        callbacks, error_codes::HTQT_BATCH_CONTINUE_ON_ERROR, htqt_error_message, htqt_error_name,
        token_context::open_token_session, BatchEncryptParams, BatchResult, CryptoCallbacksV2,
        FileEntry, RecipientEntry, HTQT_OK,
    },
    AppState,
};

/// Convert a path to its Windows 8.3 short path so it can be safely passed as a
/// CString to DLLs that use ANSI (code-page) file APIs. Falls back to the original
/// path if GetShortPathNameW fails (e.g. short-path generation disabled on that volume).
fn to_short_path(path: &str) -> String {
    extern "system" {
        fn GetShortPathNameW(lp_long: *const u16, lp_short: *mut u16, cch: u32) -> u32;
    }

    let wide: Vec<u16> = std::ffi::OsStr::new(path).encode_wide().chain(Some(0)).collect();
    let mut buf = vec![0u16; 32768];
    let len = unsafe { GetShortPathNameW(wide.as_ptr(), buf.as_mut_ptr(), buf.len() as u32) };

    if len == 0 || len as usize >= buf.len() {
        return path.to_string(); // fallback
    }

    OsString::from_wide(&buf[..len as usize])
        .into_string()
        .unwrap_or_else(|_| path.to_string())
}

#[derive(Serialize, Clone)]
pub struct EncryptProgress {
    pub current: usize,
    pub total: usize,
    pub file_name: String,
    pub file_path: String,
    pub status: String, // "processing" | "success" | "warning" | "error"
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct EncryptRequest {
    pub src_paths: Vec<String>,
    pub partner_name: String,
    pub cert_paths: Vec<String>,
}

#[derive(Serialize)]
pub struct EncryptResult {
    pub total: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

/// Batch encrypt M files × N recipients via single encHTQT_multi DLL call.
/// Progress emitted per (file, recipient) pair via cb_progress callback.
#[tauri::command]
pub async fn encrypt_batch(
    app: AppHandle,
    src_paths: Vec<String>,
    partner_name: String,
    cert_paths: Vec<String>,
    output_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<EncryptResult, String> {
    *state.is_operation_running.lock().unwrap() = true;
    let result = run_encrypt_batch(&app, &src_paths, &partner_name, &cert_paths, output_dir.as_deref(), &state).await;
    *state.is_operation_running.lock().unwrap() = false;
    result
}

async fn run_encrypt_batch(
    app: &AppHandle,
    src_paths: &[String],
    _partner_name: &str,
    cert_paths: &[String],
    output_dir_override: Option<&str>,
    state: &State<'_, AppState>,
) -> Result<EncryptResult, String> {
    // Read and validate token login state
    let (pkcs11_lib, slot_id, pin_str) = {
        let login = state.token_login.lock().unwrap();
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

    if cert_paths.is_empty() {
        return Err("No recipient certificates provided".to_string());
    }

    // Get sender's own cert DER (first cert from last scan, for SF v1 backward compat)
    let own_cert_der: Vec<u8> = {
        let scan = state.last_token_scan.lock().unwrap();
        scan.as_ref()
            .and_then(|s| s.certificates.first())
            .map(|e| e.certificate.raw_der.clone())
            .unwrap_or_default()
    };

    // Resolve output directory: use override if provided, else output_data_dir\SF\ENCRYPT
    // (flat — no partner subfolder, so "Open Folder" on the Encrypt page opens the right place)
    let output_dir_string = if let Some(dir) = output_dir_override {
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
        format!("{}\\SF\\ENCRYPT", base.trim_end_matches(['/', '\\']))
    };

    std::fs::create_dir_all(&output_dir_string)
        .map_err(|e| format!("Cannot create output directory: {}", e))?;

    let file_count = src_paths.len();
    let recip_count = cert_paths.len();
    let total_pairs = file_count * recip_count;

    if total_pairs > 10_000 {
        emit_app_log(app, "warning",
            &format!("Large batch: {} files × {} recipients = {} operations", file_count, recip_count, total_pairs));
    }
    emit_app_log(app, "info",
        &format!("Starting encryption: {} file(s) × {} recipient(s)", file_count, recip_count));

    // Use Windows short path so DLL's ANSI file APIs can resolve Unicode directory names
    let output_dir_str = to_short_path(&output_dir_string);
    // ISO date suffix: -{YYYYMMDD} — DLL uses file_id as output filename base, appends .sf
    let date_suffix = chrono::Local::now().format("-%Y%m%d").to_string();

    // Pre-compute file_ids ({stem}-{YYYYMMDD}) — DLL uses this in output filename
    let file_id_strings: Vec<String> = src_paths
        .iter()
        .map(|p| {
            let stem = Path::new(p)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".to_string());
            format!("{}{}", stem, date_suffix)
        })
        .collect();

    // Extract recipient_id (cert CN) from each cert file via cert_parser
    let recipient_id_strings: Vec<String> = cert_paths
        .iter()
        .map(|cp| {
            cert_parser::parse_cert_file(cp)
                .map(|info| info.cn)
                .unwrap_or_else(|_| {
                    Path::new(cp)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "recipient".to_string())
                })
        })
        .collect();

    // Clone data for spawn_blocking (no refs across await)
    let src_paths_owned = src_paths.to_vec();
    let cert_paths_owned = cert_paths.to_vec();
    let htqt_lib_arc = state.htqt_lib.clone();
    let app_clone = app.clone();
    let recip_ids_clone = recipient_id_strings.clone();
    let file_ids_clone = file_id_strings.clone();

    let batch_results = tokio::task::spawn_blocking(move || -> Result<Vec<BatchResult>, String> {
        // NOTE: errors returned here are caught below and emitted as app_log before propagating.
        // Build CString arrays — all must outlive the DLL call
        let input_cstrings: Vec<CString> = src_paths_owned.iter()
            .map(|p| CString::new(p.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let file_id_cstrings: Vec<CString> = file_ids_clone.iter()
            .map(|s| CString::new(s.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let cert_path_cstrings: Vec<CString> = cert_paths_owned.iter()
            .map(|p| CString::new(p.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let recip_id_cstrings: Vec<CString> = recip_ids_clone.iter()
            .map(|s| CString::new(s.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let output_dir_cstring = CString::new(output_dir_str).map_err(|e| e.to_string())?;

        let file_entries: Vec<FileEntry> = (0..file_count)
            .map(|i| FileEntry {
                input_path: input_cstrings[i].as_ptr(),
                file_id: file_id_cstrings[i].as_ptr(),
            })
            .collect();

        let recip_entries: Vec<RecipientEntry> = (0..recip_count)
            .map(|i| RecipientEntry {
                cert_path: cert_path_cstrings[i].as_ptr(),
                recipient_id: recip_id_cstrings[i].as_ptr(),
            })
            .collect();

        let params = BatchEncryptParams {
            files: file_entries.as_ptr(),
            file_count: file_count as u32,
            recipients: recip_entries.as_ptr(),
            recipient_count: recip_count as u32,
            output_dir: output_dir_cstring.as_ptr(),
            flags: HTQT_BATCH_CONTINUE_ON_ERROR,
            reserved: [ptr::null_mut(); 2],
        };

        // Open PKCS#11 session for this batch operation
        let ctx = open_token_session(
            &pkcs11_lib,
            slot_id,
            &pin_str,
            app_clone,
            own_cert_der.clone(),
            "encrypt-progress".to_string(),
        )?;

        let ctx_box = Box::new(ctx);
        let user_ctx_ptr = &*ctx_box as *const _ as *mut c_void;

        let cbs = CryptoCallbacksV2 {
            sign_fn: Some(callbacks::cb_rsa_pss_sign),
            rsa_enc_cert_fn: Some(callbacks::cb_rsa_oaep_enc_cert),
            rsa_dec_fn: Some(callbacks::cb_rsa_oaep_decrypt),
            verify_fn: Some(callbacks::cb_rsa_pss_verify),
            progress_fn: Some(callbacks::cb_progress),
            user_ctx: user_ctx_ptr,
            own_cert_der: if own_cert_der.is_empty() { ptr::null() } else { own_cert_der.as_ptr() },
            own_cert_der_len: own_cert_der.len() as u32,
            reserved: [ptr::null_mut(); 3],
        };

        let mut batch_results: Vec<BatchResult> = (0..total_pairs)
            .map(|_| BatchResult::default())
            .collect();

        let guard = htqt_lib_arc.lock().unwrap();
        match guard.as_ref() {
            None => return Err("htqt_crypto.dll not loaded".to_string()),
            Some(lib) => { lib.enc_multi(&params, &cbs, &mut batch_results)?; }
        }
        drop(guard);
        drop(ctx_box); // closes PKCS#11 session + finalizes Pkcs11

        Ok(batch_results)
    });

    // Intercept DLL-level failures: emit error progress events and return Ok(EncryptResult)
    // instead of propagating Err, so the progress panel shows error rows (not an unhandled throw).
    let batch_results = match batch_results.await {
        Ok(Ok(results)) => results,
        Ok(Err(dll_err)) => {
            emit_dll_error_as_progress(app, src_paths, &dll_err);
            return Ok(EncryptResult {
                total: src_paths.len(),
                success_count: 0,
                error_count: src_paths.len(),
                errors: vec![format!("Encryption failed: {}", dll_err)],
            });
        }
        Err(join_err) => {
            // spawn_blocking thread panic — treat same as DLL error
            let dll_err = join_err.to_string();
            emit_dll_error_as_progress(app, src_paths, &dll_err);
            return Ok(EncryptResult {
                total: src_paths.len(),
                success_count: 0,
                error_count: src_paths.len(),
                errors: vec![format!("Encryption failed: {}", dll_err)],
            });
        }
    };

    // Collect results per (file, recipient) pair, emit progress events, and log to DB
    let mut success_count = 0usize;
    let mut error_count = 0usize;
    let mut errors: Vec<String> = Vec::new();
    let total_files = src_paths.len();

    for (pair_idx, result) in batch_results.iter().enumerate() {
        let fi = result.file_index as usize;
        let ri = result.recipient_index as usize;
        let file_path_str = src_paths.get(fi).map(String::as_str).unwrap_or("?");
        let output_path = unsafe { std::ffi::CStr::from_ptr(result.output_path.as_ptr()) }
            .to_string_lossy()
            .to_string();

        let file_name = Path::new(file_path_str)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| file_path_str.to_string());

        let (status_str, error_msg) = if result.status == HTQT_OK {
            success_count += 1;
            ("success".to_string(), None)
        } else {
            error_count += 1;
            let name = htqt_error_name(result.status);
            let message = htqt_error_message(result.status);
            let detail = unsafe { std::ffi::CStr::from_ptr(result.error_detail.as_ptr()) }
                .to_string_lossy()
                .to_string();
            let error_str = if detail.is_empty() {
                format!("[{}] {}: {}", result.status, name, message)
            } else {
                format!("[{}] {}: {} — {}", result.status, name, message, detail)
            };
            let recip_id = recipient_id_strings.get(ri).map(String::as_str).unwrap_or("?");
            errors.push(format!("{}+{}: {}", file_name, recip_id, error_str));
            ("error".to_string(), Some(error_str))
        };

        // Emit per-file progress event for UI status tracking
        let _ = app.emit("encrypt-progress", EncryptProgress {
            current: pair_idx + 1,
            total: total_files,
            file_name: file_name.clone(),
            file_path: file_path_str.to_string(),
            status: status_str.clone(),
            error: error_msg.clone(),
        });

        let _ = logs_repo::insert_log(
            &state.db,
            "ENCRYPT",
            file_path_str,
            &output_path,
            None,
            &status_str,
            error_msg.as_deref(),
        )
        .await;
    }

    let total = batch_results.len();
    emit_app_log(
        app,
        if error_count == 0 { "success" } else { "warning" },
        &format!("Encryption complete: {}/{} succeeded", success_count, total),
    );
    Ok(EncryptResult { total, success_count, error_count, errors })
}

/// Emits all source files as error progress events for a DLL-level batch failure.
/// Called when the DLL itself fails before processing any file (e.g. error -33: output dir not found).
/// Returns Ok(EncryptResult) rather than Err so the progress panel shows error rows.
fn emit_dll_error_as_progress(app: &AppHandle, src_paths: &[String], dll_err: &str) {
    let error_msg = format!("Encryption failed: {}", dll_err);
    for (i, fp) in src_paths.iter().enumerate() {
        let file_name = Path::new(fp)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| fp.to_string());
        let _ = app.emit("encrypt-progress", EncryptProgress {
            current: i + 1,
            total: src_paths.len(),
            file_name,
            file_path: fp.clone(),
            status: "error".to_string(),
            error: Some(error_msg.clone()),
        });
    }
}
