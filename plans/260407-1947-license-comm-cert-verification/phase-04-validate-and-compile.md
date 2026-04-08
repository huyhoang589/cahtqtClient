# Phase 4: Validate and Compile

## Context Links
- [plan.md](./plan.md)
- [Cargo.toml](../../src-tauri/Cargo.toml)

## Overview
- **Priority:** P1
- **Status:** Complete
- **Description:** Run `cargo check` (debug + release), verify no warnings, no references to old constant, no cfg-gated signature skips.

## Requirements
### Functional
- `cargo check` passes (both debug and release profiles)
- `cargo check --release` passes (previously blocked by compile_error!)
- No grep hits for `SERVER_PUBLIC_KEY_PEM` in codebase
- No grep hits for `cfg(debug_assertions)` in license module files

### Non-functional
- No new warnings introduced

## Implementation Steps

1. Run `cargo check` in `src-tauri/`:
   ```bash
   cd src-tauri && cargo check 2>&1
   ```
2. Run `cargo check --release`:
   ```bash
   cd src-tauri && cargo check --release 2>&1
   ```
3. Grep verification:
   ```bash
   grep -r "SERVER_PUBLIC_KEY_PEM" src-tauri/src/
   grep -r "debug_assertions" src-tauri/src/license/
   ```
   Both should return empty.

4. If errors found, fix in the relevant phase file's scope.

## Todo List
- [x] `cargo check` passes (debug)
- [x] `cargo check --release` passes
- [x] No references to SERVER_PUBLIC_KEY_PEM
- [x] No cfg(debug_assertions) in license module
- [x] No new warnings

## Success Criteria
- Clean compilation on both profiles
- Release build unblocked (compile_error! removed)

## Risk Assessment
- **Low:** Compilation issues caught and fixed inline

## Security Considerations
- Release build now requires valid comm cert — no bypass path exists

## Next Steps
- Code review
- Frontend update for `no_communication_cert` status (separate task/PR)
