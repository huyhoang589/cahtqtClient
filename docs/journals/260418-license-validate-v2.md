# License Validation v2 Implementation Complete

**Date**: 2026-04-18 08:47
**Severity**: Medium
**Component**: License validation, cryptographic binding
**Status**: Resolved

## What Happened

Implemented license validation v2 to align client-side behavior with server-side v2 spec. Token serial binding strategy shifted from discrete payload field to embedded fingerprint hash, strengthening cryptographic linkage between license and device.

**Branch**: `feature/licenseValidaev2`
**Commit**: 64cfaca "feat: implement license validation v2 with token serial fingerprinting"

## The Brutal Truth

This is a breaking change, but it was intentional and necessary. Server v2 went live weeks ago; client was still stuck on v1 validation logic. The gap meant licenses issued under v2 (with token serial baked into server-side fingerprint) would fail client-side verification because client was looking for a separate `token_serial` field. Shipping old v1 code would cause production validation failures for new licenses. Had to bite the bullet and obsolete v1 entirely.

## Technical Details

**Three core changes:**

1. **`machine.rs` - Fingerprint generation now includes token serial**
   ```
   Old: SHA-256(cpu + ":" + board)
   New: SHA-256(cpu + ":" + board + ":" + token_serial)
   ```
   Signature: `get_machine_fingerprint(cpu: &str, board: &str, token_serial: &str) -> String`

2. **`payload.rs` - Struct tightened, token binding removed from payload**
   - Removed: `token_serial: Option<String>` (redundant with embedded hash)
   - Removed: `version: Option<String>` (v1/v2 distinction not needed at deserialization)
   - Added: `issued_by: String` (JWT issuer claim, required)
   - Changed: `machine_fp` and `product` from `Option<String>` → `String` (required fields)

3. **`mod.rs` - Validation logic simplified**
   - Replaced branching `if let Some(ref licensed_fp)` with direct string compare: `license_payload.machine_fp != computed_fp`
   - Deleted separate `token_serial` validation block (now impossible to decouple from fingerprint)
   - Wrapped `license.product` in `Some()` when constructing `LicenseInfo` for display

## What We Tried

Started with additive approach (support both v1 and v2 logic in parallel). Realized that was architectural confusion—server v2 already discarded v1 payload format entirely. Attempting dual support would mean maintaining two separate deserialization paths indefinitely. Chose clean break instead.

## Root Cause Analysis

Timing issue: Server shipped v2 validation months before client could follow. Client was blocked waiting on requirements clarity about how token serial should bind to fingerprint. Once spec settled (embedded in hash, not separate field), implementation was straightforward, but any delay meant longer client-server mismatch window.

## Lessons Learned

1. **Token binding via hash embedding is stronger than field separation.** Attackers can't selectively modify or replay token_serial if it's cryptographically bound to fingerprint. Trade-off: payload struct is less explicit about what's being signed.

2. **Breaking changes should be fast, not polite.** Attempted v1/v2 compatibility would've added 40+ lines of conditional logic for legacy path nobody uses anymore. Clean break saved debt and confusion.

3. **Server-first shipping creates client catching-up pressure.** Should've coordinated client implementation before server v2 went live. Document: "client updates must ship within X days of server changes for features with auth dependencies."

## Next Steps

- [x] Implement token serial binding in fingerprint hash
- [x] Update payload struct to reflect server v2 format
- [x] Simplify validation logic (remove branching)
- [x] Verify `cargo build` passes (clean)
- [x] Commit to feature branch
- [ ] PR to main (blocked on any other integration testing)
- [ ] Deploy when server v2 is confirmed stable in production

**Owner**: Implementation complete. Ready for code review and integration testing.
