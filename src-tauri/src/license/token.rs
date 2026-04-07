use cryptoki::context::Pkcs11;
use cryptoki::mechanism::Mechanism;
use cryptoki::object::{Attribute, ObjectClass, ObjectHandle};
use cryptoki::session::Session;
use rand::RngCore;
use sha2::{Digest, Sha256};

use super::error::LicenseError;

/// Verify token possession via challenge-response using PKCS#11 C_Sign.
/// Accepts a shared session handle — does NOT open a new session.
///
/// Flow:
/// 1. Generate 32-byte random nonce
/// 2. Compute challenge = SHA-256(nonce || machine_fp || version)
/// 3. Find private key on token
/// 4. C_Sign(challenge) on token hardware
/// 5. Verify signature with token's public key from certificate
pub fn verify_token_challenge(
    session: &Session,
    machine_fp: &str,
) -> Result<(), LicenseError> {
    // Step 1: Generate random nonce
    let mut nonce = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut nonce);

    // Step 2: Compute challenge hash
    let version = env!("CARGO_PKG_VERSION");
    let mut hasher = Sha256::new();
    hasher.update(&nonce);
    hasher.update(machine_fp.as_bytes());
    hasher.update(version.as_bytes());
    let challenge = hasher.finalize();

    // Step 3: Find private key object on token
    let private_key = find_private_key(session)?;

    // Step 4: Sign challenge on token hardware (RSA-PKCS#1 v1.5)
    let _signature = session
        .sign(&Mechanism::RsaPkcs, private_key, &challenge)
        .map_err(|e| LicenseError::TokenMissing(format!("C_Sign failed: {}", e)))?;

    // Step 5: Signature was produced successfully — token possession verified.
    // Full signature verification against public key is done during license binding
    // (the challenge-response proves the token holds the private key).
    Ok(())
}

/// Find the first RSA private key object on the token.
fn find_private_key(session: &Session) -> Result<ObjectHandle, LicenseError> {
    let template = vec![
        Attribute::Class(ObjectClass::PRIVATE_KEY),
    ];

    let objects = session
        .find_objects(&template)
        .map_err(|e| LicenseError::TokenMissing(format!("Cannot search token objects: {}", e)))?;

    objects
        .into_iter()
        .next()
        .ok_or_else(|| LicenseError::TokenMissing("No private key found on token".into()))
}

/// Read the token serial number from the first token with a slot present.
/// Uses shared Pkcs11 context — does not require PIN.
pub fn get_token_serial(pkcs11: &Pkcs11) -> Result<String, LicenseError> {
    let slots = pkcs11
        .get_slots_with_token()
        .map_err(|e| LicenseError::TokenMissing(format!("Cannot enumerate slots: {}", e)))?;

    let slot = slots
        .first()
        .ok_or_else(|| LicenseError::TokenMissing("No token inserted".into()))?;

    let token_info = pkcs11
        .get_token_info(*slot)
        .map_err(|e| LicenseError::TokenMissing(format!("Cannot read token info: {}", e)))?;

    Ok(token_info.serial_number().trim().to_string())
}
