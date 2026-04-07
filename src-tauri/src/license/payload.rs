use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rsa::pkcs1v15::VerifyingKey;
use rsa::signature::Verifier;
use rsa::{pkcs8::DecodePublicKey, RsaPublicKey};
use serde::Deserialize;
use sha2::Sha256;

use super::error::LicenseError;

/// Separator between payload JSON and RSA signature in license.dat
const SIG_SEPARATOR: &[u8] = b"||SIG||";

/// Server public key PEM — placeholder for development.
/// Replace with actual server public key before production release.
/// SAFETY: compile_error! prevents release builds with placeholder key.
#[cfg(not(debug_assertions))]
compile_error!("Replace SERVER_PUBLIC_KEY_PEM with actual server public key before release build. Remove this compile_error! after replacement.");

const SERVER_PUBLIC_KEY_PEM: &str = r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0000000000000000000
0000000000000000000000000000000000000000000000000000000000000000000
0000000000000000000000000000000000000000000000000000000000000000000
0000000000000000000000000000000000000000000000000000000000000000000
000000000000000000000000000000000000000000000000000000000000000AQAB
-----END PUBLIC KEY-----"#;

/// License payload JSON schema matching the 2F-HBLS spec
#[derive(Debug, Deserialize)]
pub struct LicensePayload {
    pub product: Option<String>,
    pub machine_fp: Option<String>,
    pub token_serial: Option<String>,
    pub issued_at: Option<i64>,
    pub expires_at: Option<i64>,
    pub version: Option<String>,
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

/// Verify RSA-PKCS1v15-SHA256 signature over payload using embedded server public key.
pub fn verify_license_signature(payload: &[u8], sig: &[u8]) -> Result<(), LicenseError> {
    let public_key = RsaPublicKey::from_public_key_pem(SERVER_PUBLIC_KEY_PEM)
        .map_err(|e| LicenseError::InvalidKey(format!("Server public key invalid: {}", e)))?;

    let verifying_key = VerifyingKey::<Sha256>::new(public_key);
    let signature = rsa::pkcs1v15::Signature::try_from(sig)
        .map_err(|e| LicenseError::InvalidKey(format!("Invalid signature format: {}", e)))?;

    verifying_key
        .verify(payload, &signature)
        .map_err(|_| LicenseError::Corrupted("RSA signature verification failed".into()))
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

    // Verify RSA signature if server key is configured (non-placeholder)
    // For now, structural validation only since we have a placeholder key
    Ok(())
}
