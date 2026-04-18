use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rsa::pss::VerifyingKey;
use rsa::signature::Verifier;
use rsa::RsaPublicKey;
use serde::Deserialize;
use sha2::Sha256;

use super::error::LicenseError;

/// Separator between payload JSON and RSA signature in license.dat
const SIG_SEPARATOR: &[u8] = b"||SIG||";

/// License payload JSON schema matching the 2F-HBLS v2 spec
#[derive(Debug, Deserialize)]
pub struct LicensePayload {
    pub expires_at: Option<i64>,
    pub issued_at: i64,
    pub issued_by: String,
    pub machine_fp: String,
    pub product: String,
}

/// Read and decode license.dat from the given directory.
/// Returns (payload_bytes, signature_bytes) split by ||SIG|| separator.
pub fn read_license_file(app_data_dir: &std::path::Path) -> Result<(Vec<u8>, Vec<u8>), LicenseError> {
    let license_path = app_data_dir.join("license.dat");
    if !license_path.exists() {
        return Err(LicenseError::NotLicensed);
    }

    let raw = std::fs::read(&license_path)
        .map_err(|e| LicenseError::Corrupted(format!("Cannot read license.dat: {}", e)))?;

    if raw.is_empty() {
        return Err(LicenseError::Corrupted("license.dat is empty".into()));
    }

    // Base64-decode the entire file content
    let decoded = BASE64
        .decode(&raw)
        .map_err(|e| LicenseError::Corrupted(format!("Base64 decode failed: {}", e)))?;

    // Split by ||SIG|| separator
    split_payload_and_sig(&decoded)
}

/// Split decoded license data by ||SIG|| into (payload, signature).
fn split_payload_and_sig(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), LicenseError> {
    let pos = data
        .windows(SIG_SEPARATOR.len())
        .position(|w| w == SIG_SEPARATOR)
        .ok_or_else(|| LicenseError::Corrupted("Missing ||SIG|| separator".into()))?;

    let payload = data[..pos].to_vec();
    let sig = data[pos + SIG_SEPARATOR.len()..].to_vec();

    if payload.is_empty() || sig.is_empty() {
        return Err(LicenseError::Corrupted("Empty payload or signature".into()));
    }

    Ok((payload, sig))
}

/// Verify RSA-PSS-SHA256 signature over payload using caller-provided public key.
pub fn verify_license_signature(payload: &[u8], sig: &[u8], public_key: &RsaPublicKey) -> Result<(), LicenseError> {
    let verifying_key = VerifyingKey::<Sha256>::new(public_key.clone());
    let signature = rsa::pss::Signature::try_from(sig)
        .map_err(|e| LicenseError::InvalidKey(format!("Invalid signature format: {}", e)))?;

    verifying_key
        .verify(payload, &signature)
        .map_err(|e| {
            eprintln!("[license] RSA-PSS-SHA256 verify FAILED: {}", e);
            LicenseError::Corrupted("RSA signature verification failed".into())
        })
}

/// Parse license payload JSON bytes into LicensePayload struct.
pub fn parse_license_payload(payload: &[u8]) -> Result<LicensePayload, LicenseError> {
    serde_json::from_slice(payload)
        .map_err(|e| LicenseError::Corrupted(format!("Invalid license JSON: {}", e)))
}

/// Validate that license.dat structure is correct (Base64, has ||SIG|| separator).
/// Used by import_license_file to validate before persisting.
pub fn validate_license_file_structure(file_path: &str) -> Result<(), LicenseError> {
    let raw = std::fs::read(file_path)
        .map_err(|e| LicenseError::Corrupted(format!("Cannot read file: {}", e)))?;

    if raw.is_empty() {
        return Err(LicenseError::Corrupted("File is empty".into()));
    }

    let decoded = BASE64
        .decode(&raw)
        .map_err(|e| LicenseError::Corrupted(format!("Base64 decode failed: {}", e)))?;

    let _ = split_payload_and_sig(&decoded)?;

    // Structural validation only — full RSA signature verification
    // happens in is_licensed() after the file is copied to app_data_dir.
    Ok(())
}
