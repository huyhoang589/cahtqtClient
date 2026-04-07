# Brainstorm: Credential Export Format Alignment

**Date:** 2026-04-07
**Status:** Approved → Plan

## Problem
`export_machine_credential` outputs hashed `machine_fingerprint` + extra fields (`user_name`, `app_version`). Server expects raw hardware IDs in exact spec format.

## Spec Format (Required)
```json
{
  "token_serial":   "1234567890ABCDEF",
  "cpu_id":         "BFEBFBFF000906EA",
  "board_serial":   "SN20240815XYZ01",
  "registered_at":  "2025-01-01T00:00:00Z"
}
```

## Decisions
- **Match spec exactly** — server already expects this format
- **Strict spec only** — no extra fields (user_name, app_version removed)
- **Hash stays internal** — machine_fingerprint used only for license binding verification
- **registered_at** = ISO 8601 UTC timestamp at export time

## Files to Change
| File | Change |
|------|--------|
| `src-tauri/src/license/machine.rs` | Expose raw cpu_id + board_serial |
| `src-tauri/src/commands/license.rs` | Update export output format |
| `src/types/index.ts` | Update MachineCredentialResult type |
| `src/components/license-screens.tsx` | Update field references |
| `src/pages/Settings/LicenseSection.tsx` | Update field references |

## Risk
- Low — export-only change, verification pipeline unchanged
- No breaking change to license validation
