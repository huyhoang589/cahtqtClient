pub mod error;
pub mod machine;
pub mod payload;
pub mod token;

use std::path::Path;

use rsa::{pkcs8::DecodePublicKey, traits::PublicKeyParts, RsaPublicKey};
use x509_parser::prelude::*;

use error::{LicenseError, LicenseInfo, LicenseStatus};

/// Run the full license verification pipeline.
/// Designed to be called once at startup — result cached in AppState.
///
/// Pipeline phases:
/// - Phase A: Token verification (is token present, challenge-response)
/// - Phase B: License binding (read license.dat, verify signature, check expiry/machine/token)
pub fn is_licensed(pkcs11_lib_path: &str, app_data_dir: &Path, comm_cert_path: Option<&str>) -> LicenseInfo {
    match verify_full(pkcs11_lib_path, app_data_dir, comm_cert_path) {
        Ok(info) => info,
        Err(e) => LicenseInfo {
            status: e.to_status(),
            expires_at: None,
            product: None,
        },
    }
}

/// Internal full verification — returns Result for clean error propagation.
fn verify_full(pkcs11_lib_path: &str, app_data_dir: &Path, comm_cert_path: Option<&str>) -> Result<LicenseInfo, LicenseError> {
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
    eprintln!("[license] license.dat read OK — payload {} bytes, sig {} bytes", payload_bytes.len(), sig_bytes.len());

    // Step 6: Extract public key from communication certificate and verify RSA signature
    let comm_path = comm_cert_path
        .filter(|p| !p.is_empty())
        .ok_or(LicenseError::NoCommunicationCert)?;

    // Path safety: reject relative paths and directory traversal
    let comm_path_obj = std::path::Path::new(comm_path);
    if !comm_path_obj.is_absolute() || comm_path.contains("..") {
        return Err(LicenseError::InvalidKey("Invalid communication cert path".into()));
    }

    if !comm_path_obj.exists() {
        return Err(LicenseError::NoCommunicationCert);
    }

    let cert_data = std::fs::read(comm_path)
        .map_err(|e| LicenseError::InvalidKey(format!("Cannot read communication cert: {}", e)))?;
    eprintln!("[license] comm cert read OK — {} bytes from {}", cert_data.len(), comm_path);

    let public_key = extract_public_key_from_cert(&cert_data)?;
    eprintln!("[license] public key extracted — size {} bits", public_key.size() * 8);

    payload::verify_license_signature(&payload_bytes, &sig_bytes, &public_key)?;
    eprintln!("[license] RSA signature verification PASSED");

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

/// Extract RSA public key from X.509 certificate bytes (auto-detects PEM/DER).
fn extract_public_key_from_cert(cert_data: &[u8]) -> Result<RsaPublicKey, LicenseError> {
    let is_pem = cert_data.windows(b"-----BEGIN".len()).any(|w| w == b"-----BEGIN");
    eprintln!("[license] cert format: {}", if is_pem { "PEM" } else { "DER" });

    let der_bytes: Vec<u8> = if is_pem {
        let (_, pem) = x509_parser::pem::parse_x509_pem(cert_data)
            .map_err(|e| {
                eprintln!("[license] PEM parse FAILED: {:?}", e);
                LicenseError::InvalidKey(format!("PEM parse error: {:?}", e))
            })?;
        eprintln!("[license] PEM label: {}", pem.label);
        pem.contents
    } else {
        cert_data.to_vec()
    };

    let (_, cert) = parse_x509_certificate(&der_bytes)
        .map_err(|e| {
            eprintln!("[license] X.509 parse FAILED: {:?}", e);
            LicenseError::InvalidKey(format!("Certificate parse error: {:?}", e))
        })?;

    let cn = cert.subject().iter_common_name().next()
        .and_then(|a| a.as_str().ok())
        .unwrap_or("?");
    eprintln!("[license] cert CN: {}, algo: {:?}", cn, cert.public_key().algorithm.algorithm);

    let spki_der = cert.public_key().raw.to_vec();
    RsaPublicKey::from_public_key_der(&spki_der)
        .map_err(|e| {
            eprintln!("[license] RSA key extraction FAILED: {}", e);
            LicenseError::InvalidKey(format!("Not an RSA certificate: {}", e))
        })
}
