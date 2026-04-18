use sha2::{Digest, Sha256};
use std::process::Command;

/// Collect CPU Processor ID via wmic (Windows).
/// Returns None if unavailable or placeholder value.
pub fn get_cpu_id() -> Option<String> {
    let output = Command::new("wmic")
        .args(["cpu", "get", "ProcessorId", "/value"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let value = text
        .lines()
        .find(|l| l.starts_with("ProcessorId="))
        .map(|l| l.trim_start_matches("ProcessorId=").trim().to_string())?;

    validate_hardware_id(&value)
}

/// Collect motherboard serial via wmic (Windows).
/// Returns None if unavailable or placeholder value.
pub fn get_board_serial() -> Option<String> {
    let output = Command::new("wmic")
        .args(["baseboard", "get", "SerialNumber", "/value"])
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    let value = text
        .lines()
        .find(|l| l.starts_with("SerialNumber="))
        .map(|l| l.trim_start_matches("SerialNumber=").trim().to_string())?;

    validate_hardware_id(&value)
}

/// Compute machine fingerprint: HEX(SHA-256(cpu_id:board_serial:token_serial)[0..8]) = 16 hex chars.
/// Falls back to "UNAVAIL" for missing hardware IDs.
pub fn get_machine_fingerprint(token_serial: &str) -> String {
    let cpu = get_cpu_id().unwrap_or_else(|| "UNAVAIL".to_string());
    let board = get_board_serial().unwrap_or_else(|| "UNAVAIL".to_string());

    let input = format!("{}:{}:{}", cpu, board, token_serial);
    let hash = Sha256::digest(input.as_bytes());
    // Take first 8 bytes → 16 hex chars
    hex::encode(&hash[..8])
}

/// Reject placeholder / invalid hardware ID values.
fn validate_hardware_id(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.len() < 4 {
        return None;
    }
    let lower = trimmed.to_lowercase();
    let placeholders = [
        "to be filled by o.e.m.",
        "default string",
        "not applicable",
        "none",
        "n/a",
    ];
    if placeholders.iter().any(|p| lower.contains(p)) {
        return None;
    }
    Some(trimmed.to_string())
}
