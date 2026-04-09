use std::ffi::{CString, c_void};
use std::path::Path;
use std::ptr;
use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, Manager, State};
use zeroize::Zeroizing;

use crate::{
    app_log::emit_app_log,
    cert_parser,
    comm_key_service,
    etoken::models::TokenStatus,
    htqt_ffi::{
        callbacks, error_codes::HTQT_BATCH_CONTINUE_ON_ERROR,
        token_context::open_token_session, BatchEncryptParams, BatchResult, CryptoCallbacksV2,
        FileEntry, RecipientEntry, HTQT_OK,
    },
    AppState,
};

/// Certificate info for the configured communication recipient
#[derive(serde::Serialize, Clone)]
pub struct CommunicationCertInfo {
    pub cn: String,
    pub org: Option<String>,
    pub serial: String,
    pub valid_until: String, // "YYYY-MM-DD"
    pub file_path: String,
}

/// Read saved communication key info from DB settings (no file access needed).
/// Returns null if no comm key configured.
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

    // Read saved cert metadata from DB settings (saved at SET KEY time)
    let cn = crate::db::settings_repo::get_setting(&state.db, "comm_cert_cn")
        .await.ok().flatten().unwrap_or_default();
    let org = crate::db::settings_repo::get_setting(&state.db, "comm_cert_org")
        .await.ok().flatten().filter(|v| !v.is_empty());
    let serial = crate::db::settings_repo::get_setting(&state.db, "comm_cert_serial")
        .await.ok().flatten().unwrap_or_default();
    let valid_until = crate::db::settings_repo::get_setting(&state.db, "comm_cert_valid_until")
        .await.ok().flatten().unwrap_or_default();

    // If metadata is empty (legacy), return None — user must re-set key
    if cn.is_empty() || serial.is_empty() {
        return Ok(None);
    }

    Ok(Some(CommunicationCertInfo {
        cn,
        org,
        serial,
        valid_until,
        file_path: path,
    }))
}

/// Step 1: Preview — decrypt .sf1 and return cert info WITHOUT saving.
/// Stores temp_cert_path in AppState for cleanup if user cancels.
#[tauri::command]
pub async fn preview_communication_key(
    sf1_path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CommunicationCertInfo, String> {
    // Validate token is logged in
    let (pkcs11_lib, slot_id, pin, own_cert_der) = {
        let login = state.token_login.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        if login.status != TokenStatus::LoggedIn {
            return Err("Token not logged in. Please login first.".to_string());
        }
        let pin = Zeroizing::new(
            login.get_pin().ok_or("PIN not available — re-login required")?.to_string()
        );
        let lib = login.pkcs11_lib_path.clone().unwrap_or_default();
        let slot = login.slot_id.unwrap_or(0);
        let der: Vec<u8> = {
            let scan = state.last_token_scan.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
            scan.as_ref()
                .and_then(|s| s.certificates.first())
                .map(|e| e.certificate.raw_der.clone())
                .unwrap_or_default()
        };
        (lib, slot, pin, der)
    };

    // Resolve temp dir for decrypted cert
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {}", e))?;
    let temp_dir = app_data_dir.join("DATA").join("Certs").join("partners");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Cannot create temp dir: {}", e))?;
    let temp_dir_str = temp_dir.to_string_lossy().to_string();

    // Get DLL handle
    let htqt_lib_arc = state.htqt_lib.clone();
    let sf1_path_clone = sf1_path.clone();
    let app_clone = app.clone();

    // Decrypt .sf1 in spawn_blocking
    let temp_cert_path = tokio::task::spawn_blocking(move || {
        let guard = htqt_lib_arc.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        let lib = guard.as_ref().ok_or("htqt_crypto.dll not loaded")?;
        comm_key_service::decrypt_comm_key(
            &sf1_path_clone, &temp_dir_str, lib,
            &pkcs11_lib, slot_id, &*pin, own_cert_der, app_clone,
        )
    })
    .await
    .map_err(|e| e.to_string())??;

    // Parse decrypted cert to get metadata
    let info = match cert_parser::parse_cert_file(&temp_cert_path) {
        Ok(info) => info,
        Err(e) => {
            comm_key_service::cleanup_temp_cert(&temp_cert_path);
            return Err(format!("Invalid communication key: {}", e));
        }
    };

    // Store temp_cert_path in AppState for cleanup on cancel
    {
        let mut pending = state.pending_comm_key_preview.lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        // Cleanup any previous pending preview
        if let Some(ref old) = *pending {
            comm_key_service::cleanup_temp_cert(old);
        }
        *pending = Some(temp_cert_path.clone());
    }

    let valid_until = chrono::DateTime::from_timestamp(info.valid_to, 0)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    emit_app_log(&app, "info", &format!("Communication key preview: {} ({})", info.cn, info.serial));

    Ok(CommunicationCertInfo {
        cn: info.cn,
        org: info.org,
        serial: info.serial,
        valid_until,
        file_path: sf1_path,
    })
}

/// Step 2: Confirm — save .sf1 to COMM_KEY dir + save metadata to DB + cleanup temp cert.
#[tauri::command]
pub async fn confirm_set_communication_key(
    sf1_path: String,
    cert_info_cn: String,
    cert_info_org: Option<String>,
    cert_info_serial: String,
    cert_info_valid_until: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CommunicationCertInfo, String> {
    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {}", e))?;

    // Create COMM_KEY dir and copy .sf1
    let comm_key_dir = app_data_dir.join("DATA").join("COMM_KEY");
    std::fs::create_dir_all(&comm_key_dir)
        .map_err(|e| format!("Cannot create COMM_KEY dir: {}", e))?;

    let dest = comm_key_dir.join("comm_key.sf1");
    std::fs::copy(&sf1_path, &dest)
        .map_err(|e| format!("Failed to copy .sf1: {}", e))?;

    let dest_str = dest.to_string_lossy().to_string();

    // Save settings to DB
    let pairs = [
        ("communication_cert_path", dest_str.as_str()),
        ("comm_cert_cn", &cert_info_cn),
        ("comm_cert_org", cert_info_org.as_deref().unwrap_or("")),
        ("comm_cert_serial", &cert_info_serial),
        ("comm_cert_valid_until", &cert_info_valid_until),
    ];
    for (key, value) in &pairs {
        crate::db::settings_repo::set_setting(&state.db, key, value)
            .await
            .map_err(|e| format!("Failed to save setting: {}", e))?;
    }

    // Cleanup temp cert from preview
    {
        let mut pending = state.pending_comm_key_preview.lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        if let Some(ref p) = *pending {
            comm_key_service::cleanup_temp_cert(p);
        }
        *pending = None;
    }

    // Emit event so other pages (EncryptPage) refresh
    let _ = app.emit("communication-cert-changed", ());

    emit_app_log(&app, "success",
        &format!("Communication key set: {} ({})", cert_info_cn, cert_info_serial));

    Ok(CommunicationCertInfo {
        cn: cert_info_cn,
        org: cert_info_org,
        serial: cert_info_serial,
        valid_until: cert_info_valid_until,
        file_path: dest_str,
    })
}

/// Cancel preview — cleanup temp cert without saving.
#[tauri::command]
pub async fn cancel_preview_communication_key(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut pending = state.pending_comm_key_preview.lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?;
    if let Some(ref p) = *pending {
        comm_key_service::cleanup_temp_cert(p);
    }
    *pending = None;
    Ok(())
}

/// Remove communication key — delete .sf1 + clear DB settings.
#[tauri::command]
pub async fn remove_communication_key(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Read current .sf1 path
    let comm_path = crate::db::settings_repo::get_setting(&state.db, "communication_cert_path")
        .await.ok().flatten().unwrap_or_default();

    // Delete .sf1 file from COMM_KEY dir
    if !comm_path.is_empty() && Path::new(&comm_path).exists() {
        let _ = std::fs::remove_file(&comm_path);
    }

    // Clear all comm cert settings
    let keys = [
        "communication_cert_path",
        "comm_cert_cn",
        "comm_cert_org",
        "comm_cert_serial",
        "comm_cert_valid_until",
    ];
    for key in &keys {
        crate::db::settings_repo::set_setting(&state.db, key, "")
            .await
            .map_err(|e| format!("Failed to clear setting: {}", e))?;
    }

    // Emit event so other pages refresh
    let _ = app.emit("communication-cert-changed", ());

    emit_app_log(&app, "info", "Communication key removed");
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
    // Atomically set running=true; fail if already true (TOCTOU-safe)
    if state.is_operation_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Another operation is already running".to_string());
    }

    // Wrap PIN in Zeroizing immediately — no unprotected copies survive the scope
    let pin = Zeroizing::new(pin);
    let result = run_set_communication(&app, &recipient_cert_path, &partner_name, &dest_dir, &*pin, &state).await;

    state.is_operation_running.store(false, Ordering::SeqCst);
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
        let login = state.token_login.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        if login.status != TokenStatus::LoggedIn {
            return Err("Token not logged in.".to_string());
        }
        let sender_path = login.sender_cert_path.clone()
            .ok_or("Sender certificate path not available. Re-login to the token.")?;
        // Use provided pin (already Zeroizing), fallback to stored pin — keep Zeroizing wrapping
        let use_pin: Zeroizing<String> = if !pin.is_empty() {
            Zeroizing::new(pin.to_string())
        } else {
            Zeroizing::new(login.get_pin().ok_or("PIN not available — re-login required")?.to_string())
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
        let scan = state.last_token_scan.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
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
            &*pin_str,
            app_clone,
            own_cert_der.clone(),
            "setcomm-progress".to_string(),
        )?;

        let ctx_box = Box::new(ctx);
        let user_ctx_ptr = &*ctx_box as *const _ as *mut c_void;

        let cbs = CryptoCallbacksV2 {
            sign_fn: Some(callbacks::cb_rsa_pss_sign),
            rsa_dec_fn: None, // not needed for encrypt
            progress_fn: Some(callbacks::cb_progress),
            user_ctx: user_ctx_ptr,
            own_cert_der: if own_cert_der.is_empty() { ptr::null() } else { own_cert_der.as_ptr() },
            own_cert_der_len: own_cert_der.len() as u32,
            reserved: [ptr::null_mut(); 3],
        };

        let mut batch_results: Vec<BatchResult> = vec![BatchResult::default()];

        let guard = htqt_lib_arc.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
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
        let actual_out = {
            let buf = &result.output_path;
            let nul_pos = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            let bytes = &buf[..nul_pos];
            let byte_slice = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len()) };
            String::from_utf8_lossy(byte_slice).to_string()
        };
        let final_path = if actual_out.is_empty() { out_path } else { actual_out };
        emit_app_log(app, "success", &format!("SetComm complete: {}", final_path));
        Ok(final_path)
    } else {
        Err("SetComm: no results returned".to_string())
    }
}
