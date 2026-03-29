use std::path::Path;

use cryptoki::context::Pkcs11;

use crate::etoken::models::LibraryInfo;

pub struct LibraryCandidate {
    pub vendor: String,
    pub path: String,
}

/// Auto-detect PKCS#11 middleware library on Windows.
/// Priority: user-set path → bit4xpki → bit4opki → bit4ipki → eTPKCS11 → OpenSC
pub fn auto_detect_library(user_set_path: Option<&str>) -> Option<LibraryCandidate> {
    // User-specified path takes priority
    if let Some(p) = user_set_path {
        if !p.is_empty() && Path::new(p).exists() {
            return Some(LibraryCandidate {
                vendor: "Custom".to_string(),
                path: p.to_string(),
            });
        }
    }

    let system_root = std::env::var("SystemRoot")
        .unwrap_or_else(|_| "C:\\Windows".to_string());
    let sys32 = format!("{}\\System32", system_root);

    let system32_candidates = [
        ("bit4ID",  "bit4xpki.dll"),
        ("bit4ID",  "bit4opki.dll"),
        ("bit4ID",  "bit4ipki.dll"),
        ("SafeNet", "eTPKCS11.dll"),
    ];

    for (vendor, dll) in &system32_candidates {
        let path = format!("{}\\{}", sys32, dll);
        if Path::new(&path).exists() {
            return Some(LibraryCandidate {
                vendor: vendor.to_string(),
                path,
            });
        }
    }

    // OpenSC uses a fixed install path (not in System32)
    let opensc_path = r"C:\Program Files\OpenSC Project\OpenSC\pkcs11\opensc-pkcs11.dll";
    if Path::new(opensc_path).exists() {
        return Some(LibraryCandidate {
            vendor: "OpenSC".to_string(),
            path: opensc_path.to_string(),
        });
    }

    None
}

/// Query the loaded PKCS#11 library for its version/description metadata.
pub fn get_library_info(pkcs11: &Pkcs11, vendor: &str, path: &str) -> Result<LibraryInfo, String> {
    let info = pkcs11
        .get_library_info()
        .map_err(|e| format!("Failed to get library info: {}", e))?;

    let cryptoki_ver = info.cryptoki_version();
    let library_ver = info.library_version();

    Ok(LibraryInfo {
        vendor: vendor.to_string(),
        description: info.library_description().trim().to_string(),
        path: path.to_string(),
        cryptoki_version: format!("{}.{}", cryptoki_ver.major(), cryptoki_ver.minor()),
        library_version: format!("{}.{}", library_ver.major(), library_ver.minor()),
        manufacturer_id: info.manufacturer_id().trim().to_string(),
    })
}
