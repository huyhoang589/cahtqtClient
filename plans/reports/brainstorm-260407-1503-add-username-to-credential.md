# Brainstorm: Add user_name to Machine Credential Export

**Date:** 2026-04-07
**Status:** Approved — proceed to plan

## Problem
Server expects v1 credential format with `user_name` field (CN from client certificate on PKCS#11 token). Current export omits it.

## Target Format
```json
{
  "token_serial": "1234567890ABCDEF",
  "cpu_id": "BFEBFBFF000906EA",
  "board_serial": "SN20240815XYZ01",
  "user_name": "Nguyen Van A",
  "registered_at": "2025-01-01T00:00:00Z"
}
```

## Approach
Reuse existing `certificate_reader::read_all_certificates()` which already extracts `subject_cn` from X.509 certs.

### Changes Required
1. **`commands/license.rs`** — `export_machine_credential`: open RO session, call `read_all_certificates`, take first non-CA cert's `subject_cn`, add to JSON
2. **Frontend types** — no change needed (credential result only returns `saved_path`)

### Key Decisions
- No PIN required (public objects only)
- First non-CA cert's CN used (spec says singular "Client certificate")
- Empty string fallback if no cert found (same contract as other hardware IDs)
- No new dependencies needed

### Risk: Low
All infrastructure exists. Just wiring cert reader into export command.
