---
status: complete
branch: feature/encrypt-license-func-upgrade
created: 2026-04-09
completedDate: 2026-04-09
blockedBy: []
blocks: []
---

# Communication Key (.sf1) Upgrade

## Summary
Replace plaintext certificate storage with encrypted `.sf1` communication key workflow. Certificates are never stored long-term in plaintext — only decrypted temporarily when needed (SET KEY, license validation, encryption).

## Phases

| # | Phase | Status | Files |
|---|-------|--------|-------|
| 1 | Comm key service module | complete | `comm_key_service.rs`, `lib.rs` |
| 2 | Settings: SET KEY / REMOVE KEY | complete | `communication.rs`, `CommunicationSection.tsx`, `tauri-api.ts` |
| 3 | License startup with .sf1 decrypt | complete | `license/mod.rs`, `lib.rs` |
| 4 | Encrypt per-session decrypt lifecycle | complete | `encrypt.rs`, `EncryptPage.tsx` |
| 5 | Startup orphan cleanup | complete | `lib.rs` |
| 6 | Compile check + integration test | complete | all |

## Key Design Decisions
- Reuse `decrypt_one_sfv1()` DLL call for .sf1 communication key decryption
- PKCS#11 token + PIN for authentication
- Reuse `PartnerMember` table + existing cert paths
- New `DATA/COMM_KEY/` dir for .sf1 storage
- Centralized `comm_key_service` module (DRY)
- Startup orphan cleanup for crash recovery

## Dependencies
- Existing `htqt_ffi` FFI bridge (decrypt_one_sfv1)
- PKCS#11 token integration (etoken module)
- cert_parser module for X.509 parsing

## Reference
- [Brainstorm Report](../reports/brainstorm-260409-2022-comm-key-sf1-upgrade.md)
- [Feature Spec](../../feature/4.%20encrypt-license-func-upgrade/encrypt-license-func-upgrade_v1.txt)

## Validation Log

### Session 1 — 2026-04-09
**Trigger:** Initial plan validation before implementation
**Questions asked:** 5

#### Questions & Answers

1. **[Architecture/UX]** Phase 3 defers license validation until after token login (status = Pending at startup). This means the app shows 'Pending' on every cold start until user logs into token. Is this acceptable UX?
   - Options: Defer to token login (Recommended) | Prompt PIN at startup | Cache last license result
   - **Answer:** Defer to token login
   - **Rationale:** Simplifies startup — no PIN prompt dialog needed. Pending state is acceptable since encrypt/decrypt routes already gated by per-route license checks.

2. **[Compatibility]** Phase 3 includes backward compatibility for plain cert paths (non-.sf1). Do you need to support both old plaintext cert paths AND new .sf1 paths, or is this a clean migration?
   - Options: Clean migration, .sf1 only (Recommended) | Support both formats
   - **Answer:** Clean migration, .sf1 only
   - **Rationale:** Eliminates branching logic in license verification. Users must re-set communication key after upgrade — acceptable for this release.

3. **[UX]** Phase 2 removes the cert preview step — browse .sf1 triggers SET KEY immediately. Current flow has a preview-then-save step. Is immediate set correct?
   - Options: Immediate SET KEY (Recommended) | Preview before save
   - **Answer:** Preview before save
   - **Rationale:** User wants to verify cert info (CN, serial, expiry) before committing. Two-step flow: browse .sf1 → decrypt → show preview → user confirms → save .sf1 + DB.

4. **[Architecture]** Phase 5 orphan cleanup needs DB access to check which certs are referenced. Should cleanup also protect files newer than N minutes?
   - Options: Delete all unreferenced (Recommended) | Skip files < 5 min old | Dedicated temp subdir
   - **Answer:** Delete all unreferenced
   - **Rationale:** At startup no operation is running, so all unreferenced certs are safe to delete. Simple and effective.

5. **[API]** Phase 4 removes cert_paths from the encrypt API signature (breaking change). Any other callers?
   - Options: No other callers, safe to break (Recommended) | Keep cert_paths as optional param
   - **Answer:** No other callers, safe to break
   - **Rationale:** Only EncryptPage.tsx calls encrypt_batch. Clean API simplification.

#### Confirmed Decisions
- License startup: defer to token login, show Pending status — simplicity over seamless UX
- Backward compat: clean migration, .sf1 only — no dual-format support
- SET KEY UX: two-step with preview before save — user confirms cert info
- Orphan cleanup: delete all unreferenced certs at startup — simple approach
- Encrypt API: remove cert_paths param, backend resolves internally — no other callers

#### Action Items
- [ ] Phase 2: Add preview step (browse → decrypt → show preview → confirm → save)
- [ ] Phase 3: Remove backward compat code paths for plain cert files

#### Impact on Phases
- Phase 2: Add two-step UX flow — browse .sf1 → decrypt → preview cert info → user confirms SET KEY → save
- Phase 3: Remove `.crt`/`.cer` backward compat branch in `verify_full()` — only handle `.sf1` paths
