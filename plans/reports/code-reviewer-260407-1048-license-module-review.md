# Code Review: License Module

**Date:** 2026-04-07  
**Scope:** 11 files (6 new Rust, 3 new TSX, 2 modified TS, modified Cargo.toml, lib.rs, commands/mod.rs)  
**Focus:** Security, error handling, architecture, correctness

---

## Overall Assessment

Well-structured license module with clean separation (machine fingerprint, token challenge, payload verification, commands). Follows existing codebase patterns. Several security and correctness issues need attention before production.

---

## Critical Issues

### C1. RSA signature verification silently skipped in production (mod.rs:63)

```rust
let _ = payload::verify_license_signature(&payload_bytes, &sig_bytes);
```

The `let _ =` discards the signature verification result. A crafted `license.dat` with valid JSON but forged/missing signature will pass all checks. This is the **entire security model** of the license system — without signature verification, anyone can create a license file.

**Impact:** Complete license bypass. Any user can craft a valid license.dat with arbitrary JSON payload + garbage signature.

**Fix:** Once the real server public key replaces the placeholder, this MUST become:
```rust
payload::verify_license_signature(&payload_bytes, &sig_bytes)?;
```

Add a compile-time or startup assertion that the placeholder key is not present in release builds:
```rust
#[cfg(not(debug_assertions))]
compile_error!("Replace SERVER_PUBLIC_KEY_PEM with real key before release build");
```

### C2. Placeholder public key ships in binary (payload.rs:16-22)

`SERVER_PUBLIC_KEY_PEM` contains a dummy key with all zeros. If this reaches production:
- Signature verification always fails (key is invalid RSA)
- Combined with C1, signature check is already skipped, so it has no effect at all

**Fix:** Add `#[cfg(not(debug_assertions))]` compile_error as above, or gate the entire module behind a feature flag until the real key is provided.

### C3. Token challenge-response also silently skipped (mod.rs:55)

```rust
let _ = token::verify_token_challenge(&session, &machine_fp);
```

Same pattern — the 2F (two-factor) in "2F-HBLS" is completely bypassed. Token possession is never verified.

**Fix:** Same approach as C1. Must propagate error in release builds. For dev builds, use `#[cfg(debug_assertions)]` to allow bypass.

### C4. Path traversal in `import_license_file` (commands/license.rs:169)

`file_path: String` comes directly from the frontend. While Tauri's file dialog produces safe paths, the command itself can be invoked directly via `invoke("import_license_file", { filePath: "../../etc/something" })`. The file is read via `std::fs::metadata` and `std::fs::copy` with no path sanitization.

**Impact:** An attacker who can execute JS in the webview can read file metadata and copy arbitrary files into the app data directory, or more critically, use the path to probe file existence.

**Fix:** Validate that `file_path` is an absolute path and does not contain `..` segments:
```rust
let path = std::path::Path::new(&file_path);
if !path.is_absolute() || file_path.contains("..") {
    return Err("Invalid file path".to_string());
}
```

---

## High Priority

### H1. Mutex `.unwrap()` on poisoned lock panics (commands/license.rs:54, 75, 219)

```rust
let info = _state.license_info.lock().unwrap().clone();
```

If any thread panics while holding the license_info lock, all subsequent lock attempts will panic, crashing the app. This is a recurring pattern in the codebase but adding more mutex users increases the blast radius.

**Fix:** Use `.lock().map_err(|e| format!("..."))?` or at minimum document that the lock is expected never to be held during panicking code.

### H2. `export_machine_credential` leaks hardware identifiers to disk (commands/license.rs:141-149)

The credential JSON includes raw `cpu_id` and `board_serial` — these are hardware identifiers that uniquely identify the machine. The file is written to user-chosen directory (default Desktop) with no encryption.

**Impact:** If this file is emailed or shared (intended workflow for IT to generate license), the raw hardware IDs are exposed. Only the `machine_fingerprint` hash should be needed by the server.

**Recommendation:** Consider omitting `cpu_id` and `board_serial` from the export, keeping only `machine_fingerprint`. If the server needs raw IDs, document this clearly and warn the user.

### H3. Blocking filesystem I/O in async command (commands/license.rs:154, 174, 191)

`std::fs::write`, `std::fs::metadata`, `std::fs::copy` are blocking calls inside `async fn`. In Tauri v2, commands run on the async runtime — blocking I/O can stall the thread pool.

**Fix:** Use `tokio::fs::write`, `tokio::fs::metadata`, `tokio::fs::copy` since tokio is already a dependency.

### H4. `wmic` is deprecated on newer Windows (machine.rs)

`wmic` was deprecated in Windows 10 21H1 and removed in some Windows 11 builds. The command will silently fail, causing both hardware IDs to be "UNAVAIL" and the fingerprint to be `SHA256("UNAVAIL:UNAVAIL")` — identical across all such machines.

**Impact:** All machines where wmic fails get the same fingerprint, meaning licenses become interchangeable.

**Fix:** Add PowerShell/WMI fallback:
```rust
// Fallback: Get-CimInstance Win32_Processor | Select ProcessorId
Command::new("powershell")
    .args(["-NoProfile", "-Command", "Get-CimInstance Win32_Processor | Select -Expand ProcessorId"])
```

### H5. `LicenseGate` wraps outside `AppProvider` (App.tsx:17-40)

```tsx
<LicenseGate>
  <AppProvider>
    ...
  </AppProvider>
</LicenseGate>
```

`LicenseGate` is outside `AppProvider`, but `NoLicenseScreen` calls `exportMachineCredential` which reads settings from the DB (PKCS#11 path, output dir). This works because the Tauri commands don't depend on React context — but if any future license screen needs `useAppContext()`, it will crash with a missing provider error.

**Recommendation:** Move `LicenseGate` inside `AppProvider`, or document this ordering constraint.

---

## Medium Priority

### M1. Duplicated PKCS#11 path resolution logic

The pattern to resolve `pkcs11_path` from settings appears in three places:
- `lib.rs:115-128` (startup)
- `commands/license.rs:98-106` (export_machine_credential)
- `commands/license.rs:203-210` (import_license_file)

**Fix:** Extract to a shared helper function `resolve_pkcs11_path(db: &SqlitePool) -> String`.

### M2. `license.dat` file size not bounded (payload.rs:43, commands/license.rs:174)

No upper bound check on file size before reading entire file into memory. A malicious multi-GB file would cause OOM.

**Fix:** Add a size check (e.g., 1 MB max for a license file):
```rust
if metadata.len() > 1_048_576 {
    return Err("License file too large".to_string());
}
```

### M3. Missing `machine_fp` or `token_serial` in payload silently passes (mod.rs:69-80)

If `license.machine_fp` is `None`, the machine check is skipped entirely. Same for `token_serial`. A license with neither field is valid on any machine with any token.

**Impact:** A "universal" license can be crafted by omitting both fields from the JSON payload. Whether this is intentional (perpetual/site licenses) should be documented.

### M4. NoTokenScreen polls every 2s indefinitely (license-screens.tsx:18)

The `setInterval` polls `getTokenStatus` every 2 seconds forever. On machines where the user walks away, this generates continuous PKCS#11 calls.

**Recommendation:** Add exponential backoff or a max retry count with a manual "Retry" button fallback.

### M5. `LicenseSection.tsx:56` loses product info on import

```tsx
setInfo({ status: result.status, expires_at: result.expires_at, product: info?.product ?? null });
```

After import, `product` comes from the old `info` state, not the new license. The `ImportLicenseResult` doesn't return `product`, so it's lost.

**Fix:** Either include `product` in `ImportLicenseResult` from the backend, or re-fetch via `getLicenseInfo()` after import.

---

## Low Priority

### L1. `read_first_cert_cn` defined in commands file, not in license module

`read_first_cert_cn` (commands/license.rs:225) is a utility that reads PKCS#11 cert data. It belongs in `etoken/` or `cert_parser.rs` for reuse.

### L2. Error message leaks internal detail

`LicenseError::Corrupted` messages include specific parsing errors (e.g., "Base64 decode failed: ..."). For end users this is noise; for attackers it's oracle information about the file format.

**Recommendation:** Log detailed error server-side, show generic "License file is invalid" to user.

### L3. `commands/license.rs` is 251 lines — slightly over 200-line guideline

Consider extracting the PKCS#11 path resolution helper and `read_first_cert_cn` to reduce line count.

---

## Positive Observations

- Clean module separation: error types, machine fingerprint, payload verification, token challenge are well-isolated
- `#[cfg(debug_assertions)]` bypass in `check_license` is a good pattern for dev workflow
- No PII leaks to frontend — `LicenseInfo` only contains status, expiry, and product
- Hardware ID validation rejects common OEM placeholder strings
- File structure validation before copy prevents persisting garbage to app data
- Proper use of `Mutex<LicenseInfo>` for shared state across commands
- Frontend follows existing component patterns (inline styles with CSS vars, feedback messages)

---

## Recommended Actions (Priority Order)

1. **[CRITICAL]** Add `#[cfg(not(debug_assertions))]` gate preventing release builds with placeholder key
2. **[CRITICAL]** Change `let _ =` to `?` for signature verification and token challenge in release builds
3. **[CRITICAL]** Add path validation in `import_license_file`
4. **[HIGH]** Add PowerShell fallback for machine fingerprint collection
5. **[HIGH]** Switch to `tokio::fs` for async file operations
6. **[HIGH]** Bound license file size before reading
7. **[MEDIUM]** Extract PKCS#11 path resolution to shared helper
8. **[MEDIUM]** Document whether omitting `machine_fp`/`token_serial` in payload is intentional
9. **[LOW]** Move `read_first_cert_cn` to appropriate module

---

## Unresolved Questions

1. Is the placeholder public key scenario expected to be caught by CI/CD before release, or do we need the compile-time guard?
2. Are "universal" licenses (no machine_fp, no token_serial) an intentional feature for site licenses?
3. Should `cpu_id` and `board_serial` be included in the exported machine credential, or only the hash?
4. What is the expected behavior when wmic is unavailable — should the app refuse to run or silently degrade?

---

**Status:** DONE_WITH_CONCERNS  
**Summary:** License module is well-architected but has 3 critical security gaps (signature verification silently skipped, placeholder key, no path validation) that must be fixed before any release build.  
**Concerns:** The `let _ =` pattern on signature verification means the entire license security model is currently a no-op. This is acceptable for development with placeholder keys, but needs compile-time guards to prevent accidental release.
