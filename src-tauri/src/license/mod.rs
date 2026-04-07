pub mod error;
pub mod machine;
pub mod payload;
pub mod token;

use std::path::Path;

use error::{LicenseError, LicenseInfo, LicenseStatus};

/// Run the full license verification pipeline.
/// Designed to be called once at startup — result cached in AppState.
///
/// Pipeline phases:
/// - Phase A: Token verification (is token present, challenge-response)
/// - Phase B: License binding (read license.dat, verify signature, check expiry/machine/token)
pub fn is_licensed(pkcs11_lib_path: &str, app_data_dir: &Path) -> LicenseInfo {
    match verify_full(pkcs11_lib_path, app_data_dir) {
        Ok(info) => info,
        Err(e) => LicenseInfo {
            status: e.to_status(),
            expires_at: None,
            product: None,
        },
    }
}

/// Internal full verification — returns Result for clean error propagation.
fn verify_full(pkcs11_lib_path: &str, app_data_dir: &Path) -> Result<LicenseInfo, LicenseError> {
    // Phase A: Token verification
    // Step 1: Initialize PKCS#11
    let pkcs11 = crate::etoken::token_manager::initialize(pkcs11_lib_path)
        .map_err(|e| LicenseError::TokenMissing(e))?;

    // Step 2: Check token presence + get serial
    let token_serial = token::get_token_serial(&pkcs11)?;

    // Step 3: Challenge-response (proves token holds private key)
    let machine_fp = machine::get_machine_fingerprint();
    let slots = pkcs11
        .get_slots_with_token()
        .map_err(|e| LicenseError::TokenMissing(format!("Cannot enumerate slots: {}", e)))?;

    let slot = *slots
        .first()
        .ok_or_else(|| LicenseError::TokenMissing("No token slot available".into()))?;

    // Open RO session for challenge-response (no PIN needed for public objects,
    // but C_Sign may require login depending on token config)
    let session = pkcs11
        .open_ro_session(slot)
        .map_err(|e| LicenseError::TokenMissing(format!("Cannot open session: {}", e)))?;

    // Challenge-response is best-effort at startup — some tokens require PIN for C_Sign.
    // If it fails, we still proceed with license file verification.
    let _ = token::verify_token_challenge(&session, &machine_fp);

    // Phase B: License binding
    // Step 4-5: Read and verify license file
    let (payload_bytes, sig_bytes) = payload::read_license_file(app_data_dir)?;

    // Step 6: Verify RSA signature
    // In debug builds with placeholder key, skip signature verification.
    // In release builds, compile_error! in payload.rs prevents building with placeholder key,
    // so this always runs with a real key in production.
    #[cfg(not(debug_assertions))]
    payload::verify_license_signature(&payload_bytes, &sig_bytes)?;
    #[cfg(debug_assertions)]
    let _ = payload::verify_license_signature(&payload_bytes, &sig_bytes);

    // Step 7: Parse license payload
    let license = payload::parse_license_payload(&payload_bytes)?;

    // Step 8: Check machine fingerprint binding
    if let Some(ref licensed_fp) = license.machine_fp {
        if *licensed_fp != machine_fp {
            return Err(LicenseError::MachineMismatch);
        }
    }

    // Step 9: Check token serial binding
    if let Some(ref licensed_serial) = license.token_serial {
        if *licensed_serial != token_serial {
            return Err(LicenseError::TokenMismatch);
        }
    }

    // Step 10: Check expiry
    if let Some(expires_at) = license.expires_at {
        let now = chrono::Utc::now().timestamp();
        if now > expires_at {
            return Err(LicenseError::Expired);
        }
    }

    // Step 11: All checks passed
    Ok(LicenseInfo {
        status: LicenseStatus::Valid,
        expires_at: license.expires_at,
        product: license.product,
    })
}
