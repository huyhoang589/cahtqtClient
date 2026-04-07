use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::{
    app_log::emit_app_log,
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
    pub token_serial: String,
    pub user_name: String,
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
    // Collect hardware fingerprint (only hash exported — raw IDs stay on machine)
    let machine_fp = machine::get_machine_fingerprint();

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
        // Auto-detect library path
        crate::etoken::library_detector::auto_detect_library(None)
            .map(|info| info.path)
            .unwrap_or_default()
    };

    if pkcs11_path.is_empty() {
        return Err("PKCS#11 library not found. Please configure in Settings.".to_string());
    }

    // Initialize PKCS#11 and read token info
    let pkcs11 = crate::etoken::token_manager::initialize(&pkcs11_path)
        .map_err(|e| format!("PKCS#11 init failed: {}", e))?;

    let token_serial = license::token::get_token_serial(&pkcs11)
        .map_err(|e| format!("{}", e))?;

    // Read CN from first certificate (public object, no PIN needed)
    let (_, raw_slots) = crate::etoken::token_manager::get_all_slots(&pkcs11)?;
    let user_name = if let Some(&slot) = raw_slots.first() {
        let session = crate::etoken::token_manager::open_ro_session(&pkcs11, slot)?;
        read_first_cert_cn(&session).unwrap_or_else(|| "Unknown".to_string())
    } else {
        "Unknown".to_string()
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

    // Build credential JSON
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let credential = serde_json::json!({
        "machine_fingerprint": machine_fp,
        "token_serial": token_serial,
        "user_name": user_name,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "app_version": env!("CARGO_PKG_VERSION"),
    });

    let filename = format!("machine_credential_{}.json", timestamp);
    let save_path = format!("{}/{}", output_data_dir.trim_end_matches(['/', '\\']), filename);

    std::fs::write(&save_path, serde_json::to_string_pretty(&credential).unwrap())
        .map_err(|e| format!("Failed to write credential file: {}", e))?;

    emit_app_log(&app, "success", &format!("Machine credential exported to {}", save_path));

    Ok(MachineCredentialResult {
        saved_path: save_path,
        token_serial,
        user_name,
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

    let new_info = license::is_licensed(&pkcs11_path, &app_data_dir);
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

/// Read CN from the first X.509 certificate on the token (public object).
fn read_first_cert_cn(session: &cryptoki::session::Session) -> Option<String> {
    use cryptoki::object::{Attribute, AttributeType, ObjectClass};

    let template = vec![Attribute::Class(ObjectClass::CERTIFICATE)];
    let objects = session.find_objects(&template).ok()?;
    let obj = objects.first()?;

    let attrs = session
        .get_attributes(*obj, &[AttributeType::Value])
        .ok()?;

    for attr in attrs {
        if let Attribute::Value(der) = attr {
            if let Ok((_, cert)) = x509_parser::parse_x509_certificate(&der) {
                let cn = cert
                    .subject()
                    .iter_common_name()
                    .next()
                    .and_then(|cn| cn.as_str().ok())
                    .map(|s| s.to_string());
                return cn;
            }
        }
    }
    None
}
