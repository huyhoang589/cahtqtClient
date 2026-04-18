---
title: "License Validate v2"
description: "Align client-side license module with server-side v2 spec: token serial in fingerprint, updated payload struct"
status: completed
priority: P1
effort: 30m
branch: feature/licenseValidaev2
tags: [license, rust, security]
created: 2026-04-18
completed: 2026-04-18
---

# License Validate v2

## Phases

| # | Phase | Status | File(s) |
|---|-------|--------|---------|
| 01 | [Implement v2 Changes](phase-01-implement-v2-changes.md) | ✅ Complete | machine.rs, payload.rs, mod.rs |

## Summary

Two targeted changes to align client license module with server-side v2 spec:

1. **machine.rs** — bake token serial into machine fingerprint hash
2. **payload.rs** — update `LicensePayload` struct (remove `token_serial`/`version`, add `issued_by`, make fields required)
3. **mod.rs** — update callers, remove redundant token_serial check

## Key Decisions

- Token binding moves from separate payload field → embedded in fingerprint
- Old licenses obsolete — no backward compat needed
- `get_machine_fingerprint` always requires `token_serial: &str`
- `issued_by` deserialized, not validated
- Server v2 already live — safe to ship client changes
- JWT signature is sufficient for `issued_by` trust
- `LicenseInfo` struct unchanged (no `issued_by` exposure needed)
- Grep `LicensePayload` usages before implementing (confirm 3-file scope)

## Validation Log

### Session 1 — 2026-04-18
**Trigger:** Initial plan validation before implementation
**Questions asked:** 4

#### Questions & Answers

1. **[Deployment]** The plan states old licenses are obsolete with no backward compat needed. Is server-side v2 license issuance already deployed/coordinated before this client change ships?
   - Options: Yes, server v2 already live | No, ship together atomically | No, client ships first
   - **Answer:** Yes, server v2 already live
   - **Rationale:** No deployment risk — existing clients requiring new licenses is acceptable since server already issues v2 licenses.

2. **[Assumptions]** Plan says `issued_by` is deserialized but NOT validated. Should there be any validation (e.g., expected issuer name/whitelist) or is trusting the JWT signature sufficient?
   - Options: JWT signature is enough | Add issuer whitelist check | Log/record it only
   - **Answer:** JWT signature is enough
   - **Rationale:** `issued_by` remains informational only — no validation logic needed in mod.rs.

3. **[Scope]** Plan mentions `LicensePayload` might be referenced in other modules ("verify with grep if needed"). Should we grep for all usages before implementing to avoid missing call sites?
   - Options: Yes, grep first | No, trust the plan
   - **Answer:** Yes, grep first
   - **Rationale:** Add grep step before Step 1 to confirm scope is exactly 3 files. Prevents missed call sites causing compile errors.

4. **[Architecture]** Step 3.4 wraps `license.product` in `Some()` for `LicenseInfo`. Does the `LicenseInfo` struct itself need any changes (e.g., new `issued_by` field), or is the existing struct left as-is?
   - Options: Left as-is | Add issued_by to LicenseInfo | Check and decide during impl
   - **Answer:** Left as-is
   - **Rationale:** `LicenseInfo` is a display struct only — no changes needed. Keeps scope minimal.

#### Confirmed Decisions
- Deployment: server v2 live — safe to ship
- issued_by: no validation, JWT trust is sufficient
- LicenseInfo: unchanged
- Pre-impl grep: required to confirm 3-file scope

#### Action Items
- [ ] Add grep step before Step 1 in phase-01 to verify LicensePayload call sites

#### Impact on Phases
- Phase 01: Prepend grep verification step before implementation steps
