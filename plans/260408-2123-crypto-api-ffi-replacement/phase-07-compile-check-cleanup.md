## Phase 7: Compile Check + Cleanup

### Overview
- **Priority**: High
- **Status**: complete
- **Description**: Verify all changes compile, remove dead code, run clippy.

### Related Code Files
- **All modified files from phases 1-6**
- **Check**: `src-tauri/Cargo.toml` for unused dependencies

### Implementation Steps

<!-- Updated: Validation Session 1 - Full runtime testing with real DLL (not just compile check) -->

0. **Runtime testing** (DLL is available):
   - Test encrypt with multiple files + recipients → verify .sf1 output
   - Test decrypt of .sf1 → verify output path from out_path_buf
   - Verify cert fingerprint matching works correctly

1. **Rust compile check**:
   ```bash
   cd src-tauri && cargo check 2>&1
   ```
   Fix any compilation errors.

2. **Clippy lint**:
   ```bash
   cd src-tauri && cargo clippy 2>&1
   ```
   Fix warnings (unused imports, dead code).

3. **Check for unused Cargo dependencies**:
   - If `rsa` crate only used by deleted callbacks → may still be needed by remaining ones
   - `x509-parser` — check if still used elsewhere (cert_parser module likely still uses it)
   - Don't remove deps that are used by other modules

4. **Frontend compile check**:
   ```bash
   npm run build 2>&1
   ```
   Or `npx tsc --noEmit` for type checking only.

5. **Remove any dead code**:
   - Unused `use` statements
   - Unreachable code paths
   - Old comments referencing removed callbacks

### Todo
- [x] cargo check passes
- [x] cargo clippy clean
- [x] npm run build passes (or tsc --noEmit)
- [x] No dead code warnings
- [x] Cargo.toml deps still valid

### Success Criteria
- Zero compilation errors in Rust and TypeScript
- No clippy warnings related to changes
- Clean build ready for testing with new DLL
