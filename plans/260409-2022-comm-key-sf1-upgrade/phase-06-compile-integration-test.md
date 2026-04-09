# Phase 6: Compile Check + Integration Test

## Context
- All previous phases must be complete

## Overview
- **Priority:** High
- **Status:** Complete
- **Description:** Verify all changes compile and integrate correctly.

## Implementation Steps

1. Run `cargo check` in src-tauri/ — fix all compile errors
2. Run `npm run tauri dev` — verify app launches
3. Manual integration test checklist:
   - Settings: browse .sf1 → SET KEY → verify cert info displayed
   - Settings: REMOVE KEY → verify all traces cleared
   - License: startup → verify Pending status without token
   - License: login token → verify license re-validates
   - Encrypt: add files → encrypt → verify .sf1 output
   - Verify no temp cert files left in Certs/partners/ after operations

4. Check frontend TypeScript compilation: `npx tsc --noEmit`

## Todo
- [x] `cargo check` passes
- [x] `npx tsc --noEmit` passes
- [x] Manual integration smoke test
- [x] No orphaned temp certs after operations

## Success Criteria
- Zero compile errors (Rust + TypeScript)
- All 3 flows work end-to-end
- No temp certs persist after operations
