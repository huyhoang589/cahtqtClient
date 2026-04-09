use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::{
    app_log::emit_app_log,
    etoken::{certificate_reader, token_manager},
    license::{
        self,
        error::{LicenseInfo, LicenseStatus},
        machine, payload,
    },
    AppState,
};

// ---- Response types ----------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct LicenseCheckResult {
    /// "ok" | "no_token" | "no_license" | "error"
    pub state: String,
    pub error_msg: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MachineCredentialResult {
    pub saved_path: String,
}

#[derive(Debug, Serialize)]
pub struct ImportLicenseResult {
    pub status: LicenseStatus,
    pub expires_at: Option<i64>,
}

// ---- Commands ----------------------------------------------------------------

/// Check license state — called by LicenseGate on mount.
/// Reads cached license_info from AppState.
/// In debug builds, bypasses and returns "ok" immediately.
#[tauri::command]
pub async fn check_license(_state: State<'_, AppState>) -> Result<LicenseCheckResult, String> {
    // Dev bypass: auto-pass in debug builds
    #[cfg(debug_assertions)]
    {
        return Ok(LicenseCheckResult {
            state: "ok".to_string(),
            error_msg: None,
        });
    }

    #[cfg(not(debug_assertions))]
    {
        let info = _state.license_info.lock()
            .map_err(|_| "License state unavailable".to_string())?.clone();
        let (state_str, error_msg) = match info.status {
            LicenseStatus::Valid => ("ok".to_string(), None),
            LicenseStatus::NoToken => ("no_token".to_string(), Some("Please insert your Bit4ID token.".to_string())),
            LicenseStatus::NotFound => ("no_license".to_string(), Some("No valid license found.".to_string())),
            LicenseStatus::Expired => ("error".to_string(), Some("Your license has expired. Please contact IT for renewal.".to_string())),
            LicenseStatus::TokenMismatch => ("error".to_string(), Some("The inserted token does not match this machine license.".to_string())),
            LicenseStatus::MachineMismatch => ("error".to_string(), Some("This license is not valid on this machine. Please contact IT.".to_string())),
            LicenseStatus::Corrupted => ("error".to_string(), Some("License file is invalid or has been tampered with. Please contact IT.".to_string())),
            LicenseStatus::NoCommunicationCert => ("error".to_string(), Some("Communication certificate not configured. Please import the server certificate in Settings.".to_string())),
            LicenseStatus::Pending => ("pending".to_string(), Some("License check pending — login to token to verify.".to_string())),
        };
        Ok(LicenseCheckResult {
            state: state_str,
            error_msg,
        })
    }
}

/// Get cached license info — called by Settings LicenseSection on mount.
/// Works even without license (returns status for diagnostics).
#[tauri::command]
pub async fn get_license_info(state: State<'_, AppState>) -> Result<LicenseInfo, String> {
    let info = state.license_info.lock()
        .map_err(|_| "License state unavailable".to_string())?.clone();
    Ok(info)
}

/// Export machine credential — collects hardware IDs + token info, saves JSON.
/// Does not require PIN (only reads public PKCS#11 objects).
#[tauri::command]
pub async fn export_machine_credential(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<MachineCredentialResult, String> {
    // Collect raw hardware IDs for server credential
    let cpu_id = machine::get_cpu_id().unwrap_or_default();
    let board_serial = machine::get_board_serial().unwrap_or_default();

    // Get PKCS#11 library path from settings
    let settings = crate::db::settings_repo::get_all_settings(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let settings_map: std::collections::HashMap<String, String> =
        settings.into_iter().map(|s| (s.key, s.value)).collect();

    let pkcs11_mode = settings_map.get("pkcs11_mode").cloned().unwrap_or_else(|| "auto".to_string());
    let pkcs11_path = if pkcs11_mode == "manual" {
        settings_map.get("pkcs11_manual_path").cloned().unwrap_or_default()
    } else {
        crate::etoken::library_detector::auto_detect_library(None)
            .map(|info| info.path)
            .unwrap_or_default()
    };

    if pkcs11_path.is_empty() {
        return Err("PKCS#11 library not found. Please configure in Settings.".to_string());
    }

    // Initialize PKCS#11 and get slot once — reuse for token serial + cert CN
    let pkcs11 = token_manager::initialize(&pkcs11_path)
        .map_err(|e| format!("PKCS#11 init failed: {}", e))?;

    let (_slot_infos, raw_slots) = token_manager::get_all_slots(&pkcs11)
        .map_err(|e| format!("Cannot enumerate slots: {}", e))?;
    let slot = raw_slots.first().copied()
        .ok_or_else(|| "No token inserted. Please insert your token.".to_string())?;

    // Read token serial from the resolved slot
    let token_info = pkcs11.get_token_info(slot)
        .map_err(|e| format!("Cannot read token info: {}", e))?;
    let token_serial = token_info.serial_number().trim().to_string();

    // Read user_name from first non-CA certificate's CN
    let user_name = {
        let session = token_manager::open_ro_session(&pkcs11, slot)
            .map_err(|e| format!("Cannot open session: {}", e))?;
        let certs = certificate_reader::read_all_certificates(&session, 0)
            .unwrap_or_default();
        certs.first()
            .map(|c| c.subject_cn.clone())
            .unwrap_or_default()
    };

    // Resolve output directory
    let output_data_dir = settings_map
        .get("output_data_dir")
        .filter(|v| !v.is_empty())
        .cloned()
        .unwrap_or_else(|| {
            std::env::var("USERPROFILE")
                .map(|p| format!("{}/Desktop", p))
                .unwrap_or_default()
        });

    // Build credential JSON — exact server spec format
    let now = chrono::Utc::now();
    let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
    let registered_at = now.format("%Y-%m-%d").to_string();
    let credential = serde_json::json!({
        "board_serial": board_serial,
        "cpu_id": cpu_id,
        "token_serial": token_serial,
        "user_name": user_name,
        "registered_at": registered_at,
    });

    let filename = format!("machine_credential_{}.json", timestamp);
    let save_path = format!("{}/{}", output_data_dir.trim_end_matches(['/', '\\']), filename);

    let json_str = serde_json::to_string_pretty(&credential)
        .map_err(|e| format!("Failed to serialize credential: {}", e))?;
    std::fs::write(&save_path, json_str)
        .map_err(|e| format!("Failed to write credential file: {}", e))?;

    emit_app_log(&app, "success", &format!("Machine credential exported to {}", save_path));

    Ok(MachineCredentialResult {
        saved_path: save_path,
    })
}

/// Import license file — validates structure, copies to AppData, refreshes cached state.
#[tauri::command]
pub async fn import_license_file(
    file_path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ImportLicenseResult, String> {
    // Path traversal protection: reject relative paths and paths with ".." segments
    let path = std::path::Path::new(&file_path);
    if !path.is_absolute() || file_path.contains("..") {
        return Err("Invalid file path".to_string());
    }

    // Validate file exists and is non-empty
    let metadata = std::fs::metadata(&file_path)
        .map_err(|e| format!("Cannot read file: {}", e))?;
    if metadata.len() == 0 {
        return Err("License file is empty".to_string());
    }

    // Validate license file structure (Base64 + ||SIG|| separator)
    payload::validate_license_file_structure(&file_path)
        .map_err(|e| format!("{}", e))?;

    // Copy validated file to app_data_dir/license.dat
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {}", e))?;

    let dest = app_data_dir.join("license.dat");
    std::fs::copy(&file_path, &dest)
        .map_err(|e| format!("Failed to copy license file: {}", e))?;

    emit_app_log(&app, "success", "License file imported successfully");

    // Re-run verification to update cached state
    let settings = crate::db::settings_repo::get_all_settings(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let settings_map: std::collections::HashMap<String, String> =
        settings.into_iter().map(|s| (s.key, s.value)).collect();

    let pkcs11_mode = settings_map.get("pkcs11_mode").cloned().unwrap_or_else(|| "auto".to_string());
    let pkcs11_path = if pkcs11_mode == "manual" {
        settings_map.get("pkcs11_manual_path").cloned().unwrap_or_default()
    } else {
        crate::etoken::library_detector::auto_detect_library(None)
            .map(|info| info.path)
            .unwrap_or_default()
    };

    let comm_cert_path = crate::db::settings_repo::get_setting(&state.db, "communication_cert_path")
        .await
        .ok()
        .flatten();

    let new_info = license::is_licensed(&pkcs11_path, &app_data_dir, comm_cert_path.as_deref(), None);
    let result = ImportLicenseResult {
        status: new_info.status.clone(),
        expires_at: new_info.expires_at,
    };

    // Update cached state
    if let Ok(mut cached) = state.license_info.lock() {
        *cached = new_info;
    }

    Ok(result)
}

/// Re-validate license after token login — called when .sf1 decrypt becomes possible.
/// Has access to PKCS#11 session + PIN + own_cert_der via AppState.token_login.
#[tauri::command]
pub async fn revalidate_license(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<LicenseInfo, String> {
    let settings = crate::db::settings_repo::get_all_settings(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let settings_map: std::collections::HashMap<String, String> =
        settings.into_iter().map(|s| (s.key, s.value)).collect();

    let pkcs11_mode = settings_map.get("pkcs11_mode").cloned().unwrap_or_else(|| "auto".to_string());
    let pkcs11_path = if pkcs11_mode == "manual" {
        settings_map.get("pkcs11_manual_path").cloned().unwrap_or_default()
    } else {
        crate::etoken::library_detector::auto_detect_library(None)
            .map(|info| info.path)
            .unwrap_or_default()
    };

    let comm_cert_path = crate::db::settings_repo::get_setting(&state.db, "communication_cert_path")
        .await
        .ok()
        .flatten();

    let app_data_dir = app.path().app_data_dir()
        .map_err(|e| format!("Cannot resolve app data dir: {}", e))?;

    // Build token session params from login state
    let token_session = {
        let login = state.token_login.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        if login.status != crate::etoken::models::TokenStatus::LoggedIn {
            return Err("Token not logged in — cannot revalidate license".to_string());
        }
        let pin = zeroize::Zeroizing::new(login.get_pin().ok_or("PIN not available")?.to_string());
        let lib_path = login.pkcs11_lib_path.clone().unwrap_or_default();
        let slot = login.slot_id.unwrap_or(0);
        let own_cert_der: Vec<u8> = {
            let scan = state.last_token_scan.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
            scan.as_ref()
                .and_then(|s| s.certificates.first())
                .map(|e| e.certificate.raw_der.clone())
                .unwrap_or_default()
        };

        let htqt_lib_arc = state.htqt_lib.clone();
        let temp_dir = app_data_dir.join("DATA").join("Certs").join("partners")
            .to_string_lossy().to_string();

        // We need the HtqtLib ref — must do verification inside spawn_blocking with the lock
        (lib_path, slot, pin, own_cert_der, temp_dir, htqt_lib_arc, pkcs11_path.clone())
    };

    let (ts_lib_path, ts_slot, ts_pin, ts_der, ts_temp_dir, htqt_lib_arc, pkcs11_for_license) = token_session;
    let comm_cert_path_clone = comm_cert_path.clone();
    let app_data_dir_clone = app_data_dir.clone();
    let app_clone = app.clone();

    let new_info = tokio::task::spawn_blocking(move || {
        let guard = htqt_lib_arc.lock().map_err(|e| format!("Lock poisoned: {e}"))?;
        let htqt_lib = guard.as_ref().ok_or("htqt_crypto.dll not loaded")?;

        let ts = license::TokenSessionParams {
            htqt_lib,
            pkcs11_lib: &ts_lib_path,
            slot_id: ts_slot,
            pin: &ts_pin,
            own_cert_der: ts_der,
            app: app_clone,
            temp_dir: ts_temp_dir,
        };

        Ok::<LicenseInfo, String>(license::is_licensed(
            &pkcs11_for_license,
            &app_data_dir_clone,
            comm_cert_path_clone.as_deref(),
            Some(ts),
        ))
    })
    .await
    .map_err(|e| e.to_string())??;

    // Update cached state
    if let Ok(mut cached) = state.license_info.lock() {
        *cached = new_info.clone();
    }

    emit_app_log(&app, "info", &format!("License revalidated: {:?}", new_info.status));
    Ok(new_info)
}

