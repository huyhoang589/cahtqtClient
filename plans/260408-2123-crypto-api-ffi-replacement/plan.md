---
status: complete
branch: feature/cryptoApi
created: 2026-04-08
completed: 2026-04-08
blockedBy: []
blocks: []
---

# Replace DLL FFI Layer with New htqt-api.h

## Context
Replace current DLL API (`encHTQT_multi` / `decHTQT_v2`) with new SF v1 API (`encHTQT_sf_multi` / `decrypt_one_sfv1`). Key changes: fewer callbacks (DLL handles OAEP encrypt + PSS verify internally), one output .sf1 per file (all recipients embedded), cert fingerprint matching for decrypt. Clean break, no backward compat.

## Reference
- New API header: `feature/3. cryptoApi/htqt-api.h`
- Branch: `feature/cryptoApi`

## Phases

| # | Phase | Status | Files |
|---|-------|--------|-------|
| 1 | [Update FFI types](phase-01-update-ffi-types.md) | complete | `htqt_ffi/types.rs` |
| 2 | [Update DLL loader](phase-02-update-dll-loader.md) | complete | `htqt_ffi/lib_loader.rs` |
| 3 | [Simplify callbacks](phase-03-simplify-callbacks.md) | complete | `htqt_ffi/callbacks.rs` |
| 4 | [Update encrypt command](phase-04-update-encrypt-command.md) | complete | `commands/encrypt.rs` |
| 5 | [Update decrypt command](phase-05-update-decrypt-command.md) | complete | `commands/decrypt.rs` |
| 6 | [Frontend adjustments](phase-06-frontend-adjustments.md) | complete | `tauri-api.ts`, hooks |
| 7 | [Compile check + cleanup](phase-07-compile-check-cleanup.md) | complete | All modified |

## Dependencies
- Phases 1-3: Independent, can be done in any order
- Phases 4-5: Depend on phases 1-3
- Phase 6: Depends on phases 4-5 (API shape must be final)
- Phase 7: Depends on all prior phases

## Key Decisions
- Clean break: no old .sf format support
- PKCS#11 callbacks still needed (sign_fn, rsa_dec_fn)
- DLL handles OAEP encrypt + PSS verify internally
- Results capacity: file_count (not file_count * recipient_count)
- Output naming: `{file_id}.sf1`

## Validation Log

### Session 1 — 2026-04-08
**Trigger:** Initial plan validation before implementation
**Questions asked:** 4

#### Questions & Answers

1. **[Compat]** The plan assumes a clean break with no old .sf format support. Should decrypt still handle legacy .sf files (from the old API) or only .sf1?
   - Options: Only .sf1 (Recommended) | Support both .sf and .sf1 | Deprecation period
   - **Answer:** Only .sf1
   - **Rationale:** Clean break confirmed — no legacy .sf handling needed. Simplifies decrypt command and eliminates dual-path code.

2. **[Naming]** Phase 2 suggests renaming dec_v2() to decrypt_one_sfv1() or keeping the old name. Which approach for the Rust wrapper method?
   - Options: decrypt_one_sfv1() (Recommended) | Keep dec_v2() | decrypt_sf()
   - **Answer:** decrypt_one_sfv1()
   - **Rationale:** Matches DLL symbol name exactly, making FFI layer self-documenting.

3. **[partnerName]** Phase 6 is uncertain about the decryptBatch() partnerName parameter. Does the new backend decrypt command still use partnerName for output directory naming?
   - Options: Yes, keep partnerName | No, remove it | Not sure yet
   - **Answer:** No, remove it
   - **Rationale:** New API uses out_path_buf from DLL. partnerName no longer needed — remove from frontend API and backend command.

4. **[DLL Testing]** The new DLL is not yet available for testing. How should Phase 7 handle compile verification without the actual DLL?
   - Options: Compile check only (Recommended) | Mock DLL for runtime test | DLL is available now
   - **Answer:** DLL is available now
   - **Rationale:** Full runtime testing possible in Phase 7 — no need for mocks or deferred testing.

#### Confirmed Decisions
- Clean break: only .sf1, no legacy .sf support
- Rust wrapper: rename to `decrypt_one_sfv1()`
- Frontend: remove `partnerName` from decryptBatch()
- Phase 7: full runtime testing with real DLL

#### Action Items
- [x] Phase 2: Rename wrapper method to `decrypt_one_sfv1()`
- [x] Phase 5: Remove any recipient_id/partnerName handling in backend decrypt
- [x] Phase 6: Remove `partnerName` from `decryptBatch()` in tauri-api.ts and use-decrypt.ts
- [x] Phase 7: Add full runtime encrypt/decrypt test with real DLL

#### Impact on Phases
- Phase 2: Rename `dec_v2()` → `decrypt_one_sfv1()` (confirmed, not optional)
- Phase 5: Remove partnerName/partner_name from decrypt command args
- Phase 6: Remove partnerName param from decryptBatch() and related hooks
- Phase 7: Full runtime testing (not just compile check)
