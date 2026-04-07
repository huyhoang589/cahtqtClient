---
phase: 5
title: "Integration + Wiring"
status: done
effort: 0.5h
depends_on: [1, 2, 3, 4]
completed: 2026-04-07
---

# Phase 5: Integration + Wiring

## Context Links
- [Plan](plan.md) | All prior phases
- `src-tauri/src/lib.rs` — app setup + command registration
- `src/App.tsx` — app root

## Overview
- **Priority**: P1
- **Status**: pending
- **Description**: Final wiring: register all new Rust commands, ensure AppState has license field, verify end-to-end flow from startup through Settings display.

## Key Insights
- Most wiring happens in Phase 2 (commands) and Phase 4 (App.tsx). This phase is the final checklist.
- Compile both Rust backend and React frontend together with `npm run tauri dev`

## Requirements

### Functional
- Full app launches with license gate active
- Settings page shows License section with correct status
- Export credential writes valid JSON
- Import license accepts valid .dat and refreshes status

## Related Code Files

### Files to Verify (already modified in prior phases)
- `src-tauri/Cargo.toml` — hex, base64 deps added
- `src-tauri/src/lib.rs` — license module, AppState field, startup check, command registration
- `src-tauri/src/commands/mod.rs` — license module declared
- `src/App.tsx` — LicenseGate wrapper
- `src/pages/SettingsPage.tsx` — LicenseSection added
- `src/types/index.ts` — all license types present
- `src/lib/tauri-api.ts` — all license API functions present

## Implementation Steps

### 1. Verify Cargo.toml has all deps
```toml
hex = "0.4"
base64 = "0.22"
```

### 2. Verify lib.rs complete wiring
- `pub mod license;` declared
- `license_info` in AppState
- Startup check in `setup()`
- All 4 commands in `invoke_handler`

### 3. Run full build
```bash
npm run tauri dev
```

### 4. Test scenarios
- **No token inserted**: App shows NoTokenScreen
- **Token inserted, no license.dat**: App shows NoLicenseScreen
- **Valid license.dat**: App loads normally, Settings shows "Valid" + expiry
- **Expired license**: App shows ErrorScreen with expiry message
- **Settings > Export**: Credential JSON written to output_data_dir
- **Settings > Import**: File picker → select .dat → status refreshes

### 5. Verify no regressions
- Existing Settings sections (Output Dir, Token, Communication) unchanged
- Encrypt/Decrypt flows unaffected
- Token login/logout unaffected

## Todo List
- [x] Verify all Cargo deps present
- [x] Verify lib.rs wiring complete
- [x] Run `cargo check` — no errors
- [x] Run `npm run tauri dev` — app launches
- [x] Test all 6 scenarios above
- [x] Verify no regressions in existing features

## Success Criteria
- App launches end-to-end with license gate
- All license states handled correctly
- Settings License section functional
- No regressions in existing features
- `cargo check` and frontend build both pass
