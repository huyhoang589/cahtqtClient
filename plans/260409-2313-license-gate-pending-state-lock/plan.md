---
status: complete
branch: feature/encrypt-license-func-upgrade
created: 2026-04-09
blockedBy: []
blocks: []
---

# License Gate — Lock "Pending" State

## Summary
Lock encrypt/decrypt/partners pages when license state is "pending" (`.sf1` exists but token not logged in). Only "ok" unlocks the gate. After token login + revalidation, React state refreshes and gate opens automatically.

## Phases

| # | Phase | Status | Files |
|---|-------|--------|-------|
| 1 | Lock pending state in gate + conditional UX | complete | `license-required.tsx`, `license-not-found-page.tsx` |
| 2 | Wire recheckLicense after token login | complete | `login-token-modal.tsx` |
| 3 | Compile check | complete | — |

## Key Design Decisions
- Remove "pending" from allowed states in `LicenseRequired` gate
- `LicenseNotFoundPage` shows different message for "pending" vs other states
- `LoginTokenModal` uses `useAppContext()` to call `recheckLicense()` after revalidation (Option B — no prop threading)
- ~10 lines changed across 3 files, no new components

## Dependencies
- Continues from [comm-key-sf1-upgrade](../260409-2022-comm-key-sf1-upgrade/plan.md) (complete)

## Reference
- [Feature Spec v2](../../feature/4.%20encrypt-license-func-upgrade/encrypt-license-func-upgrade_v2.txt)

## Validation Log

### Session 1 — 2026-04-09
**Trigger:** Initial plan validation before implementation
**Questions asked:** 3

#### Questions & Answers

1. **[Error UX]** When token login succeeds but revalidateLicense() fails, should the modal still close and show success?
   - Options: Close + recheck (Recommended) | Show error in modal | Close + toast warning
   - **Answer:** Show error in modal
   - **Rationale:** User should know revalidation failed and have a chance to retry, rather than landing on an ambiguous gate state.

2. **[Pending UX]** Should the 'pending' blocked page include a Login Token button directly, or just instruct users where to find it?
   - Options: Text hint only (Recommended) | Embed Login Token button
   - **Answer:** Text hint only
   - **Rationale:** Simplest approach, no extra component coupling needed. Users already know the sidebar/settings location.

3. **[Type Safety]** The plan passes `reason` as a string prop. Should it be typed to the known license states for type safety?
   - Options: String is fine (Recommended) | Use LicenseState union type
   - **Answer:** Use LicenseState union type
   - **Rationale:** Provides compile-time safety and self-documents the valid states. Small effort for better maintainability.

#### Confirmed Decisions
- **Error handling on revalidation failure:** Keep modal open with error message — user needs explicit feedback
- **Pending page UX:** Text hint only, no embedded button — KISS
- **Prop typing:** Use `LicenseState` union type for `reason` prop — compile-time safety

#### Action Items
- [ ] Update Phase 2 error handling: keep modal open on revalidation failure instead of closing
- [ ] Update Phase 1: use `LicenseState` type for `reason` prop instead of `string`

#### Impact on Phases
- Phase 1: Change `reason` prop type from `string` to `LicenseState` union type
- Phase 2: Change error handling — if `revalidateLicense()` fails, show error in modal instead of closing. Remove fire-and-forget pattern.
