# Brainstorm: Communication Key (.sf1) Upgrade

**Date:** 2026-04-09
**Branch:** feature/encrypt-license-func-upgrade
**Status:** Approved → Plan creation

## Problem Statement

Currently app stores recipient certificates as plaintext files. Upgrade requires certificates distributed as encrypted `.sf1` files ("communication keys"), decryptable only via user's PKCS#11 token. Adds security layer — certificates never stored in plaintext long-term.

## Requirements (from spec)

### A. Settings Page — SET COMMUNICATION
- Rename "Browse Certificate" → "Browse communication key"
- Browse selects `.sf1` file (encrypted communication key)
- SET KEY: PIN prompt → decrypt .sf1 → extract cert → save recipient info to DB → copy .sf1 to COMM_KEY dir → delete temp cert → status: Valid
- REMOVE KEY: Delete .sf1 from COMM_KEY + delete recipient info from DB → full reset
- Button states: SET enabled ↔ REMOVE disabled, and vice versa

### B. License
- Only works when communication key set & valid
- Decrypt .sf1 → get cert → verify license.dat signature → delete temp cert
- Happens at startup with PIN prompt

### C. Encrypt Page
- Each session: decrypt .sf1 → temp cert → encrypt files → delete temp cert
- Cert removed after ALL files processed (success or failure)

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Decrypt method | `decrypt_one_sfv1()` | .sf1 comm key is regular SF v1 encrypted file |
| Auth mechanism | PKCS#11 token + PIN | Reuse existing token integration |
| DB storage | Reuse PartnerMember table | No schema changes needed |
| File paths | Existing DATA/Certs/partners/ for temp | Keep convention; new DATA/COMM_KEY/ for .sf1 storage |
| License timing | Startup with PIN prompt | User enters PIN at launch |
| Remove scope | Full reset | Delete .sf1 + clear settings + delete PartnerMember |
| Crash safety | Cleanup orphans on startup | Delete leftover temp certs in initialize_data_directories() |

## Architecture: Service Layer (Option B — Selected)

Decrypt→temp cert→use→cleanup pattern repeats in 3 flows (SET KEY, LICENSE, ENCRYPT). Centralized `comm_key_service` module keeps it DRY.

### Core Service Functions
- `decrypt_comm_key(sf1_path, token_ctx) → Result<PathBuf>` — decrypt .sf1, return temp cert path
- `cleanup_temp_cert(cert_path)` — delete temp cert file
- `cleanup_orphaned_certs()` — startup cleanup for crash recovery
- `get_comm_key_path() → Option<PathBuf>` — get stored .sf1 path from COMM_KEY dir

### Flow A — SET KEY
```
Browse .sf1 → PIN prompt → decrypt_comm_key() → parse cert
  → save PartnerMember + settings → copy .sf1 to COMM_KEY/
  → cleanup_temp_cert() → emit event → UI updates
```

### Flow B — LICENSE (Startup)
```
Check COMM_KEY .sf1 exists → PIN prompt → decrypt_comm_key()
  → extract RSA pubkey → verify license.dat signature
  → validate payload → cleanup_temp_cert() → cache in AppState
```

### Flow C — ENCRYPT (Per Session)
```
decrypt_comm_key() → encHTQT_sf_multi(files, temp_cert)
  → finally: cleanup_temp_cert()
```

## Implementation Scope

### Rust Backend
1. New `comm-key-service.rs` — shared decrypt/cleanup logic (~100 lines)
2. `settings.rs` — `set_communication_key` / `remove_communication_key` commands
3. `license/mod.rs` — integrate comm key decrypt into startup validation
4. `encrypt.rs` — wrap encrypt with comm key lifecycle
5. `lib.rs` — add orphan cleanup + COMM_KEY dir creation at startup

### Frontend
6. Settings page — "Browse communication key" + SET/REMOVE buttons + status indicator
7. PIN prompt — reuse/create PKCS#11 PIN dialog for SET KEY and encrypt flows

### DB
- No schema changes — reuse partner_members + settings tables

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| PIN prompt at startup blocks UX | Medium | Splash screen with timeout |
| DLL not thread-safe | Low | Already handled by DLL_LOCK mutex |
| Temp cert left after crash | Medium | Startup cleanup |
| .sf1 corrupted / wrong key | Low | Clear error message + status update |
| Token not inserted at startup | Medium | Defer license validation, show warning |

## Success Criteria
- SET KEY: .sf1 → PIN → decrypt → cert info displayed → .sf1 stored in COMM_KEY
- REMOVE KEY: All traces cleared (file + DB + settings)
- LICENSE: Validates at startup using decrypted comm key cert
- ENCRYPT: Per-session decrypt→encrypt→cleanup cycle works
- CRASH: No orphaned plaintext certs after restart

## Next Steps
- Create detailed implementation plan with phases
- Implement comm-key-service module first (foundation)
- Then update settings, license, encrypt flows
- Frontend adjustments last
