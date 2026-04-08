---
title: "Replace hardcoded SERVER_PUBLIC_KEY_PEM with comm cert extraction"
description: "Extract RSA public key from communication certificate at runtime for license signature verification"
status: complete
priority: P1
effort: 2h
branch: feature/license
tags: [license, communication-cert, signature-verification]
created: 2026-04-07
completed: 2026-04-07
---

# License Signature Verification via Communication Certificate

## Background

2F-HBLS license system uses hardcoded `SERVER_PUBLIC_KEY_PEM` placeholder in `payload.rs` with `compile_error!` guard. The communication certificate (saved via `save_communication_cert`) uses same RSA keypair that signs `license.dat`. This plan replaces the hardcoded key with runtime extraction from the comm cert.

## Phases

| # | Phase | Status | Files Modified |
|---|-------|--------|----------------|
| 1 | [Update error types](./phase-01-update-error-types.md) | Complete | `error.rs` |
| 2 | [Refactor signature verification](./phase-02-refactor-signature-verification.md) | Complete | `payload.rs` |
| 3 | [Wire DB + cert extraction into is_licensed](./phase-03-wire-db-cert-into-is-licensed.md) | Complete | `mod.rs`, `lib.rs`, `commands/license.rs` |
| 4 | [Validate and compile](./phase-04-validate-and-compile.md) | Complete | None (verification only) |

## Key Dependencies

- `x509-parser` 0.16 already in Cargo.toml
- `rsa` 0.9 with `sha2` feature already available
- `extract_spki_der()` pattern exists in `htqt_ffi/callbacks.rs` (reusable approach)
- `settings_repo::get_setting(pool, "communication_cert_path")` already works
- `cert_parser::parse_cert_file()` handles PEM+DER auto-detection

## Data Flow (New)

```
is_licensed(pkcs11_path, app_data_dir, db_pool)
  -> settings_repo::get_setting(db, "communication_cert_path")
  -> read cert file bytes
  -> x509_parser::parse_x509_certificate (PEM/DER)
  -> cert.public_key().raw -> RsaPublicKey::from_public_key_der()
  -> verify_license_signature(payload, sig, &public_key)
```

## Rollback

All changes on `feature/license` branch. Revert commit if needed. No DB schema changes.

## Validation Log

### Session 1 — 2026-04-07
**Trigger:** Initial plan validation before implementation
**Questions asked:** 4

#### Questions & Answers

1. **[Dev Workflow]** Removing #[cfg(debug_assertions)] means dev builds also require a valid comm cert + license.dat pair. How should dev workflow handle this?
   - Options: Require cert in dev (Recommended) | Add env var bypass | Keep cfg(debug_assertions) skip
   - **Answer:** Require cert in dev
   - **Rationale:** Matches production behavior, catches cert/license integration issues during development. Devs import comm cert before testing license activation.

2. **[Error UX]** When comm cert exists but its RSA key doesn't match the license signing key, the plan maps this to 'Corrupted/Tampered' status. Is that the right UX message for key mismatch?
   - Options: Use 'Corrupted' as-is (Recommended) | Add 'CertKeyMismatch' variant | Use 'InvalidKey' variant
   - **Answer:** Use 'Corrupted' as-is
   - **Rationale:** Key mismatch = invalid signature = semantically equivalent to tampered. Keeps error variant count minimal without losing meaningful information.

3. **[Scope]** Frontend handling of new 'no_communication_cert' status — should it be included in this PR or deferred?
   - Options: Defer to separate PR (Recommended) | Include in this PR
   - **Answer:** Defer to separate PR
   - **Rationale:** Backend-only PR is smaller and easier to review. Frontend already handles unknown status gracefully via fallback.

4. **[Sync I/O]** The plan reads cert file synchronously via std::fs::read inside verify_full. The cert could be on a slow/network path. Acceptable?
   - Options: Sync read is fine (Recommended) | Pass cert bytes from caller
   - **Answer:** Sync read is fine
   - **Rationale:** Cert is a local file in app_data_dir copied by save_communication_cert. Network paths are not a realistic scenario.

#### Confirmed Decisions
- **Dev workflow:** No debug bypass — require valid comm cert in all build profiles
- **Error mapping:** Key mismatch uses existing Corrupted variant — no new error types for this case
- **PR scope:** Backend only — frontend `no_communication_cert` handling deferred
- **I/O strategy:** Sync fs::read in verify_full — cert always local

#### Action Items
- [ ] None — all answers confirm plan as written

#### Impact on Phases
- No phase changes required — all decisions already reflected in plan
