use serde::Serialize;
use std::fmt;

/// High-level license status for UI display
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseStatus {
    Valid,
    Expired,
    NotFound,
    NoToken,
    TokenMismatch,
    MachineMismatch,
    Corrupted,
    NoCommunicationCert,
    /// .sf1 comm key exists but no token session yet — waiting for user to login
    Pending,
}

/// License info returned to frontend — no sensitive data exposed
#[derive(Debug, Clone, Serialize)]
pub struct LicenseInfo {
    pub status: LicenseStatus,
    /// Unix timestamp (seconds), None = perpetual license
    pub expires_at: Option<i64>,
    pub product: Option<String>,
}

impl Default for LicenseInfo {
    fn default() -> Self {
        Self {
            status: LicenseStatus::NotFound,
            expires_at: None,
            product: None,
        }
    }
}

/// Internal error type for license verification pipeline
#[derive(Debug)]
pub enum LicenseError {
    TokenMissing(String),
    InvalidKey(String),
    TokenMismatch,
    MachineMismatch,
    Expired,
    NotLicensed,
    Corrupted(String),
    NoCommunicationCert,
    /// .sf1 comm key exists but token session not available — deferred to after login
    Pending,
}

impl fmt::Display for LicenseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TokenMissing(msg) => write!(f, "Please insert your Bit4ID token. {}", msg),
            Self::InvalidKey(msg) => write!(f, "License file is invalid or has been tampered with. {}", msg),
            Self::TokenMismatch => write!(f, "The inserted token does not match this machine license."),
            Self::MachineMismatch => write!(f, "This license is not valid on this machine. Please contact IT."),
            Self::Expired => write!(f, "Your license has expired. Please contact IT for renewal."),
            Self::NotLicensed => write!(f, "No valid license found. Please contact your IT department."),
            Self::Corrupted(msg) => write!(f, "License file is invalid or has been tampered with. {}", msg),
            Self::NoCommunicationCert => write!(f, "Communication certificate not configured. Please import the server certificate in Settings."),
            Self::Pending => write!(f, "License check pending — login to token to verify."),
        }
    }
}

impl LicenseError {
    /// Convert to LicenseStatus for frontend display
    pub fn to_status(&self) -> LicenseStatus {
        match self {
            Self::TokenMissing(_) => LicenseStatus::NoToken,
            Self::InvalidKey(_) => LicenseStatus::Corrupted,
            Self::TokenMismatch => LicenseStatus::TokenMismatch,
            Self::MachineMismatch => LicenseStatus::MachineMismatch,
            Self::Expired => LicenseStatus::Expired,
            Self::NotLicensed => LicenseStatus::NotFound,
            Self::Corrupted(_) => LicenseStatus::Corrupted,
            Self::NoCommunicationCert => LicenseStatus::NoCommunicationCert,
            Self::Pending => LicenseStatus::Pending,
        }
    }
}
