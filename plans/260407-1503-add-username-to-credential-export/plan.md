---
title: "Add user_name to credential export"
description: "Add user_name field (cert CN) to machine credential JSON for server v1 spec"
status: complete
priority: P2
effort: 30m
branch: feature/license
tags: [license, credential, pkcs11]
created: 2026-04-07
---

# Add user_name to Machine Credential Export

## Goal
Align credential export with server v1 spec by adding `user_name` field — the Common Name (CN) from the client certificate stored on the PKCS#11 hardware token.

## Target Output
```json
{
  "token_serial": "1234567890ABCDEF",
  "cpu_id": "BFEBFBFF000906EA",
  "board_serial": "SN20240815XYZ01",
  "user_name": "Nguyen Van A",
  "registered_at": "2025-01-01T00:00:00Z"
}
```

## Phases

| # | Phase | Status | Effort |
|---|-------|--------|--------|
| 1 | [Rust: wire cert CN into credential export](phase-01-wire-cert-cn-into-credential-export.md) | Complete | 30m |

## Key Decisions
- Reuse `certificate_reader::read_all_certificates()` — no new deps
- First non-CA cert's `subject_cn` used as `user_name`
- Empty string fallback if no cert found (same contract as `cpu_id`/`board_serial`)
- No PIN required — public cert objects only
- No frontend changes needed (`MachineCredentialResult` only returns `saved_path`)

## Dependencies
- PKCS#11 library configured in settings (already required for token_serial)
- Token inserted with at least one non-CA certificate

## References
- Brainstorm: `plans/reports/brainstorm-260407-1503-add-username-to-credential.md`
- Spec: `feature/1. license/ClientMachineCredentialFormat.v1.txt`

## Validation Log

### Session 1 — 2026-04-07
**Trigger:** Initial plan validation before implementation
**Questions asked:** 3

#### Questions & Answers

1. **[Architecture]** The plan calls `get_slots_with_token()` a second time (it's already called inside `get_token_serial()`). Should we refactor to reuse the slot, or keep the duplicate call for simplicity?
   - Options: Keep duplicate call (Recommended) | Refactor to reuse slot
   - **Answer:** Refactor to reuse slot
   - **Rationale:** Avoids redundant PKCS#11 call; requires extracting slot from `get_token_serial` flow so cert reading can reuse it.

2. **[Assumptions]** If the token has multiple non-CA certificates, the plan picks the first one's CN. What if different certs have different CNs?
   - Options: First non-CA cert is fine (Recommended) | Filter by specific criteria | Concatenate all CNs
   - **Answer:** First non-CA cert is fine
   - **Rationale:** Tokens typically have one user cert. First match sufficient for MVP.

3. **[Contract]** The plan uses empty string as fallback when no cert is found. The server spec may expect null or omission instead. Which contract?
   - Options: Empty string (Recommended) | Null / omit field | Error / abort export
   - **Answer:** Empty string
   - **Rationale:** Consistent with existing cpu_id/board_serial fallback pattern.

#### Confirmed Decisions
- **Slot reuse**: Refactor to extract slot from `get_token_serial` flow — reuse for cert reading
- **Multi-cert**: First non-CA cert's CN is sufficient
- **Fallback**: Empty string, consistent with existing pattern

#### Action Items
- [ ] Refactor `export_machine_credential` to get slot once and pass to both `get_token_serial` and cert reading

#### Impact on Phases
- Phase 1: Implementation steps need updating — extract slot first, pass to `get_token_serial` and `open_ro_session` instead of calling `get_slots_with_token()` twice
