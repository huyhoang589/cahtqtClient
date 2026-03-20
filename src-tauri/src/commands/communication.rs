use std::ffi::{CString, c_void};
use std::path::Path;
use std::ptr;

use tauri::{AppHandle, Manager, State};

use crate::{
    app_log::emit_app_log,
    cert_parser,
    etoken::models::TokenStatus,
    htqt_ffi::{
        callbacks, error_codes::HTQT_BATCH_CONTINUE_ON_ERROR,
        token_context::open_token_session, BatchEncryptParams, BatchResult, CryptoCallbacksV2,
        FileEntry, RecipientEntry, HTQT_OK,
    },
    AppState,
};

/// Certificate info for the configured communication recipient
#[derive(serde::Serialize)]
pub struct CommunicationCertInfo {
    pub cn: String,
    pub org: Option<String>,
    pub serial: String,
    pub valid_until: String, // "YYYY-MM-DD"
    pub file_path: String,
}

/// Read the saved communication cert config from DB, parse cert, return info.
/// Returns null if no cert configured or file missing on disk.
#[tauri::command]
pub async fn get_communication_cert(
    state: State<'_, AppState>,
) -> Result<Option<CommunicationCertInfo>, String> {
    let path = crate::db::settings_repo::get_setting(&state.db, "communication_cert_path")
        .await
        .map_err(|e| e.to_string())?;

    let path = match path {
        Some(p) if !p.is_empty() => p,
        _ => return Ok(None),
    };

    // Graceful: if file was externally deleted, return None (not error)
    if !std::path::Path::new(&path).exists() {
        return Ok(None);
    }

    let info = crate::cert_parser::parse_cert_file(&path)
        .map_err(|e| format!("Failed to parse communication cert: {}", e))?;

    let valid_until = chrono::DateTime::from_timestamp(info.valid_to, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    Ok(Some(CommunicationCertInfo {
        cn: info.cn,
        org: info.org,
        serial: info.serial,
        valid_until,
        file_path: path,
    }))
}

/// Browse a cert file, copy to DATA/Certs/partners/<SERIAL>.crt, save path to DB.
#[tauri::command]
pub async fn save_communication_cert(
    cert_path: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<CommunicationCertInfo, String> {
    // Parse cert to get serial + metadata
    let info = crate::cert_parser::parse_cert_file(&cert_path)
        .map_err(|e| format!("Failed to parse certificate: {}", e))?;

    // Compute destination: DATA/Certs/partners/<SERIAL>.crt
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {}", e))?;
    let dest_dir = app_data_dir.join("DATA").join("Certs").join("partners");
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Cannot create cert directory: {}", e))?;

    let dest_path = dest_dir.join(format!("{}.crt", info.serial));
    std::fs::copy(&cert_path, &dest_path)
        .map_err(|e| format!("Failed to copy certificate: {}", e))?;

    let dest_str = dest_path.to_string_lossy().to_string();

    // Save path to DB
    crate::db::settings_repo::set_setting(&state.db, "communication_cert_path", &dest_str)
        .await
        .map_err(|e| format!("Failed to save setting: {}", e))?;

    let valid_until = chrono::DateTime::from_timestamp(info.valid_to, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    crate::app_log::emit_app_log(
        &app, "success",
        &format!("Communication cert saved: {} ({})", info.cn, info.serial),
    );

    Ok(CommunicationCertInfo {
        cn: info.cn,
        org: info.org,
        serial: info.serial,
        valid_until,
        file_path: dest_str,
    })
}

/// Clear the communication cert config from DB (does NOT delete file from disk).
#[tauri::command]
pub async fn clear_communication_cert(
    state: State<'_, AppState>,
) -> Result<(), String> {
    crate::db::settings_repo::set_setting(&state.db, "communication_cert_path", "")
        .await
        .map_err(|e| format!("Failed to clear setting: {}", e))?;
    Ok(())
}

/// Encrypt sender's certificate to a single partner member using encHTQT_multi.
/// Output: {dest_dir}/SetComm_{partner_name}_{DDMMYYYY}.sf
#[tauri::command]
pub async fn set_communication(
    recipient_cert_path: String,
    partner_name: String,
    dest_dir: String,
    pin: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, String> {
    // Guard: another operation running
    {
        let mut running = state.is_operation_running.lock().unwrap();
        if *running {
            return Err("Another operation is in progress.".to_string());
        }
        *running = true;
    }

    let result = run_set_communication(&app, &recipient_cert_path, &partner_name, &dest_dir, &pin, &state).await;

    *state.is_operation_running.lock().unwrap() = false;
    result
}

async fn run_set_communication(
    app: &AppHandle,
    recipient_cert_path: &str,
    partner_name: &str,
    dest_dir: &str,
    pin: &str,
    state: &State<'_, AppState>,
) -> Result<String, String> {
    // Read sender_cert_path and login state
    let (pkcs11_lib, slot_id, pin_str, sender_cert_path) = {
        let login = state.token_login.lock().unwrap();
        if login.status != TokenStatus::LoggedIn {
            return Err("Token not logged in.".to_string());
        }
        let sender_path = login.sender_cert_path.clone()
            .ok_or("Sender certificate path not available. Re-login to the token.")?;
        // Use provided pin, fallback to stored pin
        let use_pin = if !pin.is_empty() {
            pin.to_string()
        } else {
            login.get_pin().ok_or("PIN not available — re-login required")?.to_string()
        };
        (
            login.pkcs11_lib_path.clone().unwrap_or_default(),
            login.slot_id.unwrap_or(0),
            use_pin,
            sender_path,
        )
    };

    // Validate sender cert exists
    if !Path::new(&sender_cert_path).exists() {
        return Err(format!("Sender cert not found at: {}", sender_cert_path));
    }

    // Get own cert DER from scan cache
    let own_cert_der: Vec<u8> = {
        let scan = state.last_token_scan.lock().unwrap();
        scan.as_ref()
            .and_then(|s| s.certificates.first())
            .map(|e| e.certificate.raw_der.clone())
            .unwrap_or_default()
    };

    // Create dest_dir
    std::fs::create_dir_all(dest_dir)
        .map_err(|e| format!("Cannot create output directory: {}", e))?;

    // Compute output file_id: SetComm_{safe_name}_{DDMMYYYY}
    let date_str = chrono::Local::now().format("%d%m%Y").to_string();
    let safe_name = partner_name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>();
    let file_id = format!("SetComm_{}_{}", safe_name, date_str);
    let out_path = format!("{}/{}.sf", dest_dir.trim_end_matches(['/', '\\']), file_id);

    // Extract recipient_id (CN) from cert
    let recipient_id = cert_parser::parse_cert_file(recipient_cert_path)
        .map(|info| info.cn)
        .unwrap_or_else(|_| {
            Path::new(recipient_cert_path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "recipient".to_string())
        });

    let sender_cert_path_owned = sender_cert_path.clone();
    let recipient_cert_path_owned = recipient_cert_path.to_string();
    let dest_dir_owned = dest_dir.to_string();
    let file_id_owned = file_id.clone();
    let recipient_id_owned = recipient_id.clone();
    let htqt_lib_arc = state.htqt_lib.clone();
    let app_clone = app.clone();

    emit_app_log(app, "info", &format!("SetComm: encrypting sender cert → {}", recipient_id));

    let batch_results = tokio::task::spawn_blocking(move || -> Result<Vec<BatchResult>, String> {
        let input_cs = CString::new(sender_cert_path_owned.as_str()).map_err(|e| e.to_string())?;
        let file_id_cs = CString::new(file_id_owned.as_str()).map_err(|e| e.to_string())?;
        let cert_cs = CString::new(recipient_cert_path_owned.as_str()).map_err(|e| e.to_string())?;
        let recip_id_cs = CString::new(recipient_id_owned.as_str()).map_err(|e| e.to_string())?;
        let out_dir_cs = CString::new(dest_dir_owned.as_str()).map_err(|e| e.to_string())?;

        let file_entries = vec![FileEntry {
            input_path: input_cs.as_ptr(),
            file_id: file_id_cs.as_ptr(),
        }];
        let recip_entries = vec![RecipientEntry {
            cert_path: cert_cs.as_ptr(),
            recipient_id: recip_id_cs.as_ptr(),
        }];

        let params = BatchEncryptParams {
            files: file_entries.as_ptr(),
            file_count: 1,
            recipients: recip_entries.as_ptr(),
            recipient_count: 1,
            output_dir: out_dir_cs.as_ptr(),
            flags: HTQT_BATCH_CONTINUE_ON_ERROR,
            reserved: [ptr::null_mut(); 2],
        };

        let ctx = open_token_session(
            &pkcs11_lib,
            slot_id,
            &pin_str,
            app_clone,
            own_cert_der.clone(),
            "setcomm-progress".to_string(),
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

        let mut batch_results: Vec<BatchResult> = vec![BatchResult::default()];

        let guard = htqt_lib_arc.lock().unwrap();
        match guard.as_ref() {
            None => return Err("htqt_crypto.dll not loaded".to_string()),
            Some(lib) => { lib.enc_multi(&params, &cbs, &mut batch_results)?; }
        }
        drop(guard);
        drop(ctx_box);

        Ok(batch_results)
    })
    .await
    .map_err(|e| e.to_string())??;

    if let Some(result) = batch_results.first() {
        if result.status != HTQT_OK {
            let msg = format!("SetComm failed: {}",
                crate::htqt_ffi::htqt_error_display(result.status));
            emit_app_log(app, "error", &msg);
            return Err(msg);
        }
        // Get actual output path from DLL result
        let actual_out = unsafe { std::ffi::CStr::from_ptr(result.output_path.as_ptr()) }
            .to_string_lossy()
            .to_string();
        let final_path = if actual_out.is_empty() { out_path } else { actual_out };
        emit_app_log(app, "success", &format!("SetComm complete: {}", final_path));
        Ok(final_path)
    } else {
        Err("SetComm: no results returned".to_string())
    }
}
