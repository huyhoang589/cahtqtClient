# Phase 01: Wire Certificate CN into Credential Export

## Context Links
- Parent plan: [plan.md](plan.md)
- Brainstorm: `plans/reports/brainstorm-260407-1503-add-username-to-credential.md`
- Spec: `feature/1. license/ClientMachineCredentialFormat.v1.txt`

## Overview
- **Priority:** P2
- **Status:** Complete
- **Description:** Add `user_name` field to `export_machine_credential` by reading CN from the first non-CA certificate on the PKCS#11 token.

## Key Insights
- `certificate_reader::read_all_certificates(session, slot_id)` already extracts `subject_cn` from X.509 certs
- `token_manager::open_ro_session(pkcs11, slot)` opens session without PIN
- `get_token_serial` internally calls `pkcs11.get_slots_with_token()` — we need to do the same to get the raw `Slot` for opening a session
- Public cert objects are readable without login
<!-- Updated: Validation Session 1 - Refactor to extract slot once, reuse for both get_token_serial and cert reading -->

## Requirements
- **Functional:** Credential JSON must include `"user_name"` with CN from first non-CA cert
- **Non-functional:** No PIN prompt, no new dependencies, empty string fallback

## Architecture
```
export_machine_credential
  ├── get_cpu_id()          (existing)
  ├── get_board_serial()    (existing)
  ├── initialize(pkcs11)    (existing)
  ├── get_token_serial()    (existing)
  ├── get_slots_with_token()  ← NEW: get raw Slot
  ├── open_ro_session()       ← NEW: open session
  ├── read_all_certificates() ← NEW: read certs
  ├── first cert subject_cn   ← NEW: extract CN
  └── build JSON with user_name field
```

## Related Code Files
### Modify
- `src-tauri/src/commands/license.rs` — `export_machine_credential` function (lines 82-152)

### Read Only (context)
- `src-tauri/src/etoken/certificate_reader.rs` — `read_all_certificates()` API
- `src-tauri/src/etoken/token_manager.rs` — `open_ro_session()`, `get_all_slots()`
- `src-tauri/src/license/token.rs` — `get_token_serial()` pattern for slot access

## Implementation Steps

1. **Add imports** in `commands/license.rs`:
   - `use crate::etoken::{certificate_reader, token_manager};`

2. **Refactor slot extraction** — get slot once before `get_token_serial()`:
   ```rust
   // Get slot once, reuse for token_serial and cert CN
   let slots = pkcs11.get_slots_with_token()
       .map_err(|e| format!("Cannot enumerate slots: {}", e))?;
   let slot = slots.first().copied();
   ```
   Then refactor `get_token_serial()` usage to pass the slot (or keep existing call if `get_token_serial` encapsulates its own slot lookup — check actual signature).

3. **After `get_token_serial()` call**, add cert CN extraction using the already-resolved slot:
   ```rust
   // Read user_name from first non-CA certificate's CN
   let user_name = match slot {
       Some(slot) => {
           let session = token_manager::open_ro_session(&pkcs11, slot)
               .map_err(|e| format!("Cannot open session: {}", e))?;
           let certs = certificate_reader::read_all_certificates(&session, 0)
               .unwrap_or_default();
           certs.first()
               .map(|c| c.subject_cn.clone())
               .unwrap_or_default()
       }
       None => String::new(),
   };
   ```
   <!-- Updated: Validation Session 1 - Refactored to extract slot once and reuse -->

3. **Add `user_name` to credential JSON** (line ~132):
   ```rust
   let credential = serde_json::json!({
       "token_serial": token_serial,
       "cpu_id": cpu_id,
       "board_serial": board_serial,
       "user_name": user_name,
       "registered_at": registered_at,
   });
   ```

4. **Compile check:** `cargo check` in `src-tauri/`

## Todo List
- [x] Add etoken imports to commands/license.rs
- [x] Refactor slot extraction — get slot once, reuse for serial + cert CN
- [x] Add cert CN extraction after token_serial
- [x] Add user_name to credential JSON
- [x] Run `cargo check` — verify no compile errors
- [ ] Run `npm run build` — verify full build passes

## Success Criteria
- Exported JSON contains `user_name` field with cert CN value
- Empty string when no non-CA cert found (not crash/error)
- No PIN prompt during export
- `cargo check` and `npm run build` pass

## Risk Assessment
- **Low:** Token with zero non-CA certs → empty string (handled)
- **Low:** Second `get_slots_with_token()` call is cheap and idempotent
- **None:** No API contract changes (frontend only sees `saved_path`)

## Security Considerations
- CN read from public cert objects — no PIN/login escalation
- No sensitive data exposure beyond what's already in the credential

## Next Steps
- After implementation: run compile tests
- Code review for the change
- Update docs if credential format is documented
