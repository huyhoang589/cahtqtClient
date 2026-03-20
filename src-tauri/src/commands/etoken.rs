use std::path::Path;

use cryptoki::{mechanism::MechanismType, session::UserType};
use secrecy::Secret;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use zeroize::Zeroizing;

use crate::{
    app_log::emit_app_log,
    db::settings_repo,
    etoken::{
        certificate_exporter, certificate_reader, library_detector, token_manager,
        models::{
            LibraryInfo, MechanismDetail, SenderCertExportResult, TokenLoginState, TokenScanResult,
            TokenStatus,
        },
    },
    AppState,
};

// ---- Response types ----------------------------------------------------------

#[derive(Serialize)]
pub struct LoginTokenResult {
    pub cert_cn: String,
    pub status: String, // "logged_in"
}

#[derive(Serialize)]
pub struct TokenStatusResponse {
    pub status: String, // "disconnected" | "connected" | "logged_in"
    pub cert_cn: Option<String>,
    pub dll_found: bool,
    pub dll_required_path: String,
}

// ---- token_scan --------------------------------------------------------------

/// Full token scan: auto-detect or use override path, enumerate slots+tokens, read all certs.
/// Result is cached in AppState.last_token_scan.
/// Always resets token_login: Connected if tokens found, Disconnected otherwise.
#[tauri::command]
pub async fn token_scan(
    lib_path_override: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<TokenScanResult, String> {
    {
        let running = state.is_operation_running.lock().unwrap();
        if *running {
            return Err("Cannot scan token while encryption/decryption is in progress".to_string());
        }
    }

    // Step 1: DLL check — verify htqt.dll exists and loads cleanly
    let dll_required_path = state.dll_required_path.clone();
    if !dll_required_path.is_empty() {
        let p = std::path::Path::new(&dll_required_path);
        if p.exists() {
            match crate::htqt_ffi::HtqtLib::load(&dll_required_path) {
                Ok(lib) => {
                    *state.htqt_lib.lock().unwrap() = Some(lib);
                    emit_app_log(&app, "success",
                        &format!("htqt.dll loaded: {}", dll_required_path));
                }
                Err(e) => {
                    emit_app_log(&app, "error",
                        &format!("htqt.dll found but failed to load: {}", e));
                }
            }
        } else {
            emit_app_log(&app, "error",
                &format!("required lib path: {}", dll_required_path));
        }
    }

    // Clear previous cache
    *state.last_token_scan.lock().unwrap() = None;

    // Load saved library path from settings
    let custom_path = settings_repo::get_all_settings(&state.db)
        .await
        .ok()
        .and_then(|settings| {
            settings
                .into_iter()
                .find(|s| s.key == "pkcs11_library_path")
        })
        .map(|s| s.value);

    let resolved_path = lib_path_override.or(custom_path);

    let result = tokio::task::spawn_blocking(move || run_full_scan(resolved_path.as_deref()))
        .await
        .map_err(|e| e.to_string())??;

    emit_app_log(
        &app,
        "info",
        &format!("Token scan complete: {} cert(s) found", result.certificates.len()),
    );
    emit_app_log(
        &app,
        "info",
        &format!("pkcs11_lib_path: {}", result.library.path),
    );
    if let Some(token) = result.tokens.first() {
        emit_app_log(
            &app,
            "info",
            &format!("token slot ID: {}", token.slot_id),
        );
    }
    // Emit mechanism support lines with key size info
    for m in &result.mechanisms {
        let (mark, level) = if m.supported { ("✓", "info") } else { ("✗ MISSING", "warning") };
        emit_app_log(&app, level, &format!("  {} {} ({}–{} bits)", mark, m.name, m.min_key_bits, m.max_key_bits));
    }

    // Cache result in AppState
    *state.last_token_scan.lock().unwrap() = Some(result.clone());

    // Reset token_login state based on scan result (always force re-login after scan)
    {
        let mut login = state.token_login.lock().unwrap();
        if !result.tokens.is_empty() {
            // Connected: scan succeeded + token present — user must re-login
            login.status = TokenStatus::Connected;
            login.pkcs11_lib_path = Some(result.library.path.clone());
            login.slot_id = result.tokens.first().map(|t| t.slot_id as u32);
            login.cert_cn = None;
            login.pin = None; // Force re-login even if previously LoggedIn
        } else {
            // No tokens found — reset to Disconnected
            *login = TokenLoginState::default();
        }
    }

    Ok(result)
}

/// Blocking inner scan — runs all PKCS#11 operations synchronously.
fn run_full_scan(user_path: Option<&str>) -> Result<TokenScanResult, String> {
    let candidate = library_detector::auto_detect_library(user_path).ok_or_else(|| {
        "No eToken middleware detected. Please install bit4ID Universal Middleware and try again."
            .to_string()
    })?;

    let pkcs11 = token_manager::initialize(&candidate.path)?;
    let lib_info = library_detector::get_library_info(&pkcs11, &candidate.vendor, &candidate.path)?;

    let (slots, raw_slots) = token_manager::get_all_slots(&pkcs11)?;
    let tokens = token_manager::get_token_infos(&pkcs11, &slots, &raw_slots);

    // Query mechanism details from first slot (app requires OAEP + PSS)
    let mechanisms: Vec<MechanismDetail> = if let Some(&raw_slot) = raw_slots.first() {
        let supported_list = pkcs11.get_mechanism_list(raw_slot).unwrap_or_default();
        let targets = [
            (MechanismType::RSA_PKCS_OAEP, "RSA_PKCS_OAEP", "PKCS#1 v2.1"),
            (MechanismType::RSA_PKCS_PSS,  "RSA_PKCS_PSS",  "PKCS#1 v2.1"),
        ];
        targets.iter().map(|(mech_type, name, standard)| {
            if !supported_list.contains(mech_type) {
                return MechanismDetail {
                    name: name.to_string(),
                    pkcs_standard: standard.to_string(),
                    min_key_bits: 0,
                    max_key_bits: 0,
                    flags: vec![],
                    supported: false,
                };
            }
            match pkcs11.get_mechanism_info(raw_slot, *mech_type) {
                Ok(info) => {
                    let mut flags = Vec::new();
                    if info.encrypt()  { flags.push("encrypt".into()); }
                    if info.decrypt()  { flags.push("decrypt".into()); }
                    if info.sign()     { flags.push("sign".into()); }
                    if info.verify()   { flags.push("verify".into()); }
                    if info.wrap()     { flags.push("wrap".into()); }
                    if info.unwrap()   { flags.push("unwrap".into()); }
                    MechanismDetail {
                        name: name.to_string(),
                        pkcs_standard: standard.to_string(),
                        min_key_bits: info.min_key_size() as u64,
                        max_key_bits: info.max_key_size() as u64,
                        flags,
                        supported: true,
                    }
                }
                Err(_) => MechanismDetail {
                    name: name.to_string(),
                    pkcs_standard: standard.to_string(),
                    min_key_bits: 0,
                    max_key_bits: 0,
                    flags: vec![],
                    supported: false,
                },
            }
        }).collect()
    } else {
        vec![]
    };

    let mut cert_entries = Vec::new();
    for (i, slot_info) in slots.iter().enumerate().filter(|(_, s)| s.token_present) {
        let raw_slot = raw_slots[i];
        match pkcs11.open_ro_session(raw_slot) {
            Ok(session) => {
                let certs = certificate_reader::read_all_certificates(&session, slot_info.slot_id)
                    .unwrap_or_default();
                for cert in certs {
                    cert_entries.push(crate::etoken::models::TokenCertEntry {
                        slot_id: slot_info.slot_id,
                        certificate: cert,
                    });
                }
            }
            Err(_) => continue,
        }
    }

    let _ = pkcs11.finalize();

    Ok(TokenScanResult {
        library: lib_info,
        slots,
        tokens,
        certificates: cert_entries,
        mechanisms,
        scan_time: chrono::Utc::now().to_rfc3339(),
        error: None,
    })
}

// ---- token_get_library_info --------------------------------------------------

/// Quick library detection without full scan — used on Settings page load.
#[tauri::command]
pub async fn token_get_library_info(
    state: State<'_, AppState>,
) -> Result<LibraryInfo, String> {
    let custom_path = settings_repo::get_all_settings(&state.db)
        .await
        .ok()
        .and_then(|s| s.into_iter().find(|kv| kv.key == "pkcs11_library_path"))
        .map(|kv| kv.value);

    tokio::task::spawn_blocking(move || {
        let candidate = library_detector::auto_detect_library(custom_path.as_deref())
            .ok_or_else(|| "No eToken middleware detected.".to_string())?;
        let pkcs11 = token_manager::initialize(&candidate.path)?;
        let info = library_detector::get_library_info(&pkcs11, &candidate.vendor, &candidate.path)?;
        let _ = pkcs11.finalize();
        Ok::<LibraryInfo, String>(info)
    })
    .await
    .map_err(|e| e.to_string())?
}

// ---- login_token -------------------------------------------------------------

/// Verify PIN via PKCS#11 C_Login, then store verified state in AppState.token_login.
/// PIN stays in Zeroizing<String> for later use by encrypt_batch/decrypt_batch.
#[tauri::command]
pub async fn login_token(
    pin: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<LoginTokenResult, String> {
    if *state.is_operation_running.lock().unwrap() {
        return Err("Cannot login while operation is in progress".to_string());
    }

    // Read pkcs11_lib_path + slot_id from last_token_scan
    let (pkcs11_lib_path, slot_id_u64) = {
        let scan = state.last_token_scan.lock().unwrap();
        let scan = scan.as_ref().ok_or("No token scan result — scan token first")?;
        let lib_path = scan.library.path.clone();
        let slot = scan.tokens.first().ok_or("No tokens found — scan token first")?;
        (lib_path, slot.slot_id)
    };
    let slot_id_u32 = slot_id_u64 as u32;

    let pin_clone = pin.clone();
    let lib_path_clone = pkcs11_lib_path.clone();

    let (cert_cn, verified_lib_path, verified_slot) = tokio::task::spawn_blocking(move || {
        let pkcs11 = token_manager::initialize(&lib_path_clone)?;

        let raw_slots = pkcs11
            .get_slots_with_token()
            .map_err(|e| format!("Slot enumeration failed: {}", e))?;
        let raw_slot = raw_slots
            .get(slot_id_u32 as usize)
            .ok_or("Slot index out of range")?;

        // RW session required for C_Login
        let session = pkcs11
            .open_rw_session(*raw_slot)
            .map_err(|e| format!("Failed to open RW session: {}", e))?;

        // C_Login — CKR_USER_ALREADY_LOGGED_IN treated as success
        let auth_pin = Secret::new(pin_clone.clone());
        match session.login(UserType::User, Some(&auth_pin)) {
            Ok(()) => {}
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("CKR_USER_ALREADY_LOGGED_IN") {
                    // treat as success — token already authenticated
                } else if msg.contains("CKR_PIN_INCORRECT") {
                    return Err("Incorrect PIN (CKR_PIN_INCORRECT)".to_string());
                } else if msg.contains("CKR_PIN_LOCKED") {
                    return Err("Token locked — contact administrator".to_string());
                } else {
                    return Err(format!("Login failed: {}", msg));
                }
            }
        }

        // Read cert CN for display (uses slot_id as index, same as scan)
        let certs = certificate_reader::read_all_certificates(&session, slot_id_u32 as u64)
            .unwrap_or_default();
        let cert_cn = certs.first().map(|c| c.subject_cn.clone()).unwrap_or_default();

        let _ = session.logout();
        drop(session); // close session before C_Finalize — PKCS#11 spec requires no open sessions
        let _ = pkcs11.finalize();

        Ok::<(String, String, u32), String>((cert_cn, lib_path_clone, slot_id_u32))
    })
    .await
    .map_err(|e| e.to_string())??;

    // Save sender cert DER to DATA/Certs/sender/sender.crt (always overwrite)
    let sender_cert_path = {
        let scan = state.last_token_scan.lock().unwrap();
        if let Some(der) = scan.as_ref()
            .and_then(|s| s.certificates.first())
            .map(|e| e.certificate.raw_der.clone())
        {
            let sender_dir = app
                .path()
                .app_data_dir()
                .ok()
                .map(|p| p.join("DATA").join("Certs").join("sender"));
            if let Some(dir) = sender_dir {
                let _ = std::fs::create_dir_all(&dir);
                let dest = dir.join("sender.crt");
                if std::fs::write(&dest, &der).is_ok() {
                    Some(dest.to_string_lossy().to_string())
                } else {
                    None
                }
            } else { None }
        } else { None }
    };

    // Store verified state in AppState
    {
        let mut login = state.token_login.lock().unwrap();
        login.status = TokenStatus::LoggedIn;
        login.pkcs11_lib_path = Some(verified_lib_path);
        login.slot_id = Some(verified_slot);
        login.cert_cn = Some(cert_cn.clone());
        login.sender_cert_path = sender_cert_path.clone();
        login.pin = Some(Zeroizing::new(pin)); // stored until logout or app restart
    }

    if let Some(ref p) = sender_cert_path {
        emit_app_log(&app, "info", &format!("Sender cert saved: {}", p));
    }
    emit_app_log(&app, "success", &format!("Token authenticated: {}", cert_cn));
    Ok(LoginTokenResult { cert_cn, status: "logged_in".to_string() })
}

// ---- logout_token ------------------------------------------------------------

/// Clear login state — zeroizes PIN from memory, resets to Disconnected.
#[tauri::command]
pub async fn logout_token(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.token_login.lock().unwrap().logout();
    emit_app_log(&app, "info", "Token logged out");
    Ok(())
}

// ---- get_token_status --------------------------------------------------------

/// Poll current token status for UI. Returns status, cert_cn, and dll_found flag.
#[tauri::command]
pub async fn get_token_status(state: State<'_, AppState>) -> Result<TokenStatusResponse, String> {
    let dll_found = state.htqt_lib.lock().unwrap().is_some();
    let login = state.token_login.lock().unwrap();
    let status = match login.status {
        TokenStatus::Disconnected => "disconnected",
        TokenStatus::Connected => "connected",
        TokenStatus::LoggedIn => "logged_in",
    };
    Ok(TokenStatusResponse {
        status: status.to_string(),
        cert_cn: login.cert_cn.clone(),
        dll_found,
        dll_required_path: state.dll_required_path.clone(),
    })
}

// ---- token_export_sender_cert ------------------------------------------------

/// Export a selected certificate from the scan cache as the sender certificate.
#[tauri::command]
pub async fn token_export_sender_cert(
    cert_object_id: String,
    slot_id: u64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SenderCertExportResult, String> {
    let cert = {
        let guard = state.last_token_scan.lock().unwrap();
        let scan = guard
            .as_ref()
            .ok_or("No scan result available. Please scan token first.")?;
        scan.certificates
            .iter()
            .find(|e| e.slot_id == slot_id && e.certificate.object_id == cert_object_id)
            .map(|e| e.certificate.clone())
            .ok_or_else(|| "Certificate not found in scan result.".to_string())?
    };

    let sender_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("DATA")
        .join("Certs")
        .join("sender");
    std::fs::create_dir_all(&sender_dir).map_err(|e| e.to_string())?;

    let saved_path =
        certificate_exporter::export_cert_file(&cert.raw_der, &sender_dir, &cert.subject_cn)?;

    let pairs = [
        ("sender_cert_path", saved_path.clone()),
        ("sender_cn", cert.subject_cn.clone()),
        ("sender_email", cert.subject_email.clone()),
        ("sender_org", cert.subject_org.clone()),
        ("sender_serial", cert.serial_number.clone()),
        ("sender_valid_until", cert.valid_until.clone()),
    ];
    for (key, value) in &pairs {
        settings_repo::set_setting(&state.db, key, value)
            .await
            .map_err(|e| e.to_string())?;
    }

    emit_app_log(
        &app,
        "success",
        &format!("Sender certificate saved: {}", cert.subject_cn),
    );

    Ok(SenderCertExportResult {
        saved_path,
        display_name: cert.subject_cn,
        email: cert.subject_email,
        organization: cert.subject_org,
        serial: cert.serial_number,
        valid_until: cert.valid_until,
    })
}

// ---- token_set_library_path --------------------------------------------------

/// Set a custom PKCS#11 library path. Validates file exists + can be loaded.
#[tauri::command]
pub async fn token_set_library_path(
    path: String,
    state: State<'_, AppState>,
) -> Result<LibraryInfo, String> {
    if !Path::new(&path).exists() {
        return Err(format!("File not found: {}", path));
    }
    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if !["dll", "so", "dylib"].contains(&ext.as_str()) {
        return Err("File must be a .dll, .so, or .dylib library".to_string());
    }

    let path_clone = path.clone();
    let info = tokio::task::spawn_blocking(move || {
        let pkcs11 = token_manager::initialize(&path_clone)?;
        let info = library_detector::get_library_info(&pkcs11, "Custom", &path_clone)?;
        let _ = pkcs11.finalize();
        Ok::<LibraryInfo, String>(info)
    })
    .await
    .map_err(|e| e.to_string())??;

    settings_repo::set_setting(&state.db, "pkcs11_library_path", &path)
        .await
        .map_err(|e| e.to_string())?;

    Ok(info)
}

// ---- token_clear_sender_cert -------------------------------------------------

/// Clear sender certificate from settings (does NOT delete the file from disk).
#[tauri::command]
pub async fn token_clear_sender_cert(state: State<'_, AppState>) -> Result<(), String> {
    let keys = [
        "sender_cert_path",
        "sender_cn",
        "sender_email",
        "sender_org",
        "sender_serial",
        "sender_valid_until",
    ];
    for key in &keys {
        settings_repo::set_setting(&state.db, key, "")
            .await
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
