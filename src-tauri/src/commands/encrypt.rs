use std::ffi::{CString, OsString, c_void};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::Path;
use std::ptr;
use std::sync::atomic::Ordering;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use zeroize::Zeroizing;

use crate::{
    app_log::emit_app_log,
    cert_parser,
    comm_key_service::{self, TempCertGuard},
    db::{logs_repo, settings_repo},
    etoken::models::TokenStatus,
    htqt_ffi::{
        callbacks,
        error_codes::{HTQT_BATCH_CONTINUE_ON_ERROR, HTQT_BATCH_OVERWRITE_OUTPUT},
        htqt_error_message, htqt_error_name,
        token_context::open_token_session, BatchEncryptParams, BatchResult, CryptoCallbacksV2,
        FileEntry, RecipientEntry, HTQT_OK,
    },
    AppState,
};

/// Convert a path to its Windows 8.3 short path so it can be safely passed as a
/// CString to DLLs that use ANSI (code-page) file APIs.
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

#[derive(Serialize)]
pub struct EncryptResult {
    pub total: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

/// Batch encrypt M files × N recipients via single encHTQT_multi DLL call.
/// Backend resolves comm key cert internally — no cert_paths from frontend.
#[tauri::command]
pub async fn encrypt_batch(
    app: AppHandle,
    src_paths: Vec<String>,
    output_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<EncryptResult, String> {
    // Atomically set running=true; fail if already true (TOCTOU-safe)
    if state.is_operation_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Another operation is already running".to_string());
    }
    let result = run_encrypt_batch(&app, &src_paths, output_dir.as_deref(), &state).await;
    state.is_operation_running.store(false, Ordering::SeqCst);
    result
}

async fn run_encrypt_batch(
    app: &AppHandle,
    src_paths: &[String],
    output_dir_override: Option<&str>,
    state: &State<'_, AppState>,
) -> Result<EncryptResult, String> {
    // Read and validate token login state
    let (pkcs11_lib, slot_id, pin_str) = {
        let login = state.token_login.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        if login.status != TokenStatus::LoggedIn {
            return Err("Token not logged in — login via Settings first".to_string());
        }
        let pin = Zeroizing::new(login.get_pin().ok_or("PIN not available — re-login required")?.to_string());
        (
            login.pkcs11_lib_path.clone().unwrap_or_default(),
            login.slot_id.unwrap_or(0),
            pin,
        )
    };

    // Read comm key .sf1 path from settings
    let comm_key_path = settings_repo::get_setting(&state.db, "communication_cert_path")
        .await
        .ok()
        .flatten()
        .filter(|v| !v.is_empty())
        .ok_or("Communication key not set — configure in Settings first")?;

    if !Path::new(&comm_key_path).exists() {
        return Err("Communication key file not found — re-set in Settings".to_string());
    }

    // Get sender's own cert DER (first cert from last scan)
    let own_cert_der: Vec<u8> = {
        let scan = state.last_token_scan.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        scan.as_ref()
            .and_then(|s| s.certificates.first())
            .map(|e| e.certificate.raw_der.clone())
            .unwrap_or_default()
    };

    // Resolve output directory
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
    if file_count == 0 {
        return Err("No files to encrypt".to_string());
    }

    emit_app_log(app, "info", &format!("Starting encryption: {} file(s)", file_count));

    let output_dir_str = to_short_path(&output_dir_string);
    let date_suffix = chrono::Local::now().format("-%Y%m%d").to_string();

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

    // Resolve temp dir for .sf1 decrypted cert
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {}", e))?;
    let temp_dir = app_data_dir.join("DATA").join("Certs").join("partners")
        .to_string_lossy().to_string();

    // Clone data for spawn_blocking
    let src_paths_owned = src_paths.to_vec();
    let htqt_lib_arc = state.htqt_lib.clone();
    let app_clone = app.clone();
    let file_ids_clone = file_id_strings.clone();
    let comm_key_path_clone = comm_key_path.clone();
    let pkcs11_lib_clone = pkcs11_lib.clone();

    let batch_results = tokio::task::spawn_blocking(move || -> Result<Vec<BatchResult>, String> {
        // Step 1: Decrypt .sf1 comm key → temp cert path
        // decrypt_comm_key opens its own PKCS#11 session for RSA-OAEP decrypt, which
        // closes automatically when done. Sequential: session 1 closes before session 2.
        let temp_cert_path = {
            let guard = htqt_lib_arc.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
            let lib = guard.as_ref().ok_or("htqt_crypto.dll not loaded")?;
            comm_key_service::decrypt_comm_key(
                &comm_key_path_clone, &temp_dir, lib,
                &pkcs11_lib_clone, slot_id, &*pin_str, own_cert_der.clone(), app_clone.clone(),
            )?
            // guard dropped here — htqt_lib unlocked before enc_multi
        };

        // RAII guard ensures cleanup even on panic
        let _cert_guard = TempCertGuard { path: Some(temp_cert_path.clone()) };

        // Extract recipient_id (CN) from decrypted cert
        let recipient_id = cert_parser::parse_cert_file(&temp_cert_path)
            .map(|info| info.cn)
            .unwrap_or_else(|_| "recipient".to_string());

        // Build CString arrays
        let input_cstrings: Vec<CString> = src_paths_owned.iter()
            .map(|p| CString::new(p.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let file_id_cstrings: Vec<CString> = file_ids_clone.iter()
            .map(|s| CString::new(s.as_str()).map_err(|e| e.to_string()))
            .collect::<Result<_, _>>()?;
        let cert_cstring = CString::new(temp_cert_path.as_str()).map_err(|e| e.to_string())?;
        let recip_id_cstring = CString::new(recipient_id.as_str()).map_err(|e| e.to_string())?;
        let output_dir_cstring = CString::new(output_dir_str).map_err(|e| e.to_string())?;

        let file_entries: Vec<FileEntry> = (0..file_count)
            .map(|i| FileEntry {
                input_path: input_cstrings[i].as_ptr(),
                file_id: file_id_cstrings[i].as_ptr(),
            })
            .collect();

        // Single recipient from decrypted comm key
        let recip_entries = vec![RecipientEntry {
            cert_path: cert_cstring.as_ptr(),
            recipient_id: recip_id_cstring.as_ptr(),
        }];

        let params = BatchEncryptParams {
            files: file_entries.as_ptr(),
            file_count: file_count as u32,
            recipients: recip_entries.as_ptr(),
            recipient_count: 1,
            output_dir: output_dir_cstring.as_ptr(),
            flags: HTQT_BATCH_CONTINUE_ON_ERROR | HTQT_BATCH_OVERWRITE_OUTPUT,
            reserved: [ptr::null_mut(); 2],
        };

        // Step 2: Open PKCS#11 session for encrypt signing (session 1 already closed)
        let ctx = open_token_session(
            &pkcs11_lib_clone,
            slot_id,
            &*pin_str,
            app_clone,
            own_cert_der.clone(),
            "encrypt-progress".to_string(),
        )?;

        let ctx_box = Box::new(ctx);
        let user_ctx_ptr = &*ctx_box as *const _ as *mut c_void;

        let cbs = CryptoCallbacksV2 {
            sign_fn: Some(callbacks::cb_rsa_pss_sign),
            rsa_dec_fn: None,
            progress_fn: Some(callbacks::cb_progress),
            user_ctx: user_ctx_ptr,
            own_cert_der: if own_cert_der.is_empty() { ptr::null() } else { own_cert_der.as_ptr() },
            own_cert_der_len: own_cert_der.len() as u32,
            reserved: [ptr::null_mut(); 3],
        };

        let mut batch_results: Vec<BatchResult> = (0..file_count)
            .map(|_| BatchResult::default())
            .collect();

        // Step 3: Encrypt — single lock acquisition
        let guard = htqt_lib_arc.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        match guard.as_ref() {
            None => return Err("htqt_crypto.dll not loaded".to_string()),
            Some(lib) => { lib.enc_multi(&params, &cbs, &mut batch_results)?; }
        }
        drop(guard);
        drop(ctx_box);

        // _cert_guard drops here → cleanup_temp_cert
        Ok(batch_results)
    });

    // Intercept DLL-level failures
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

    // Collect results per file, emit progress, log to DB
    let mut success_count = 0usize;
    let mut error_count = 0usize;
    let mut errors: Vec<String> = Vec::new();
    let total_files = src_paths.len();

    for (file_idx, result) in batch_results.iter().enumerate() {
        let fi = result.file_index as usize;
        let file_path_str = src_paths.get(fi).map(String::as_str).unwrap_or("?");
        let output_path = {
            let buf = &result.output_path;
            let nul_pos = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            let bytes = &buf[..nul_pos];
            // SAFETY: reinterpret [i8] as [u8] — same size/layout, then lossy-convert
            let byte_slice = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len()) };
            String::from_utf8_lossy(byte_slice).to_string()
        };

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
            let detail = {
                let buf = &result.error_detail;
                let nul_pos = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
                let bytes = &buf[..nul_pos];
                let byte_slice = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len()) };
                String::from_utf8_lossy(byte_slice).to_string()
            };
            let error_str = if detail.is_empty() {
                format!("[{}] {}: {}", result.status, name, message)
            } else {
                format!("[{}] {}: {} — {}", result.status, name, message, detail)
            };
            errors.push(format!("{}: {}", file_name, error_str));
            ("error".to_string(), Some(error_str))
        };

        let _ = app.emit("encrypt-progress", EncryptProgress {
            current: file_idx + 1,
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
