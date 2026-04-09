## Code Review: Communication Key (.sf1) Upgrade

**Branch:** feature/encrypt-license-func-upgrade | **Files:** 15 | **Focus:** security, correctness, state management

### Critical

1. **PIN not zeroized in encrypt.rs:95** - `pin_str` is plain `String`, not `Zeroizing<String>`. PIN persists in heap after function returns. Compare with `communication.rs:86` which correctly wraps in `Zeroizing`. Fix: `let pin = Zeroizing::new(login.get_pin()...to_string());`

2. **PIN not zeroized in license.rs:283** - Same issue in `revalidate_license`. Plain `String` holds PIN through `spawn_blocking` closure and beyond.

### High

3. **Double PKCS#11 sessions in encrypt_batch** - `decrypt_comm_key` (line 184) opens session 1, then line 232 opens session 2 for signing. Hardware tokens with limited session slots may reject the second open. Consider reusing session or closing first explicitly before opening second.

4. **Unsafe `from_utf8_unchecked` on DLL output** (encrypt.rs:311, communication.rs:452) - DLL output buffer assumed valid UTF-8. Windows DLLs using ANSI codepage APIs may write non-UTF-8 bytes, causing UB. Use `from_utf8_lossy` or validate.

### Medium

5. **Pending preview temp cert leaks on app exit** - If user previews comm key then quits without confirm/cancel, temp cert stays on disk. `pending_comm_key_preview` not cleaned at shutdown. Mitigated by startup orphan cleanup but cert is exposed until next launch.

6. **htqt_lib double-lock pattern in encrypt.rs:261-263** - Guard dropped then re-acquired. Between drop and re-lock, another thread could set lib to None. Pattern works but fragile; comment explains intent but a single-lock approach would be safer.

### Positive

- TempCertGuard RAII pattern: solid panic-safe cleanup
- AtomicBool compare_exchange for operation running: proper TOCTOU prevention
- Path traversal checks on license import and comm cert paths
- Zeroizing PIN in communication.rs preview/set_communication flows
- Startup orphan cleanup with DB cross-reference is thorough

### Summary

Two critical PIN-in-cleartext issues in encrypt and license revalidation paths. The PKCS#11 double-session risk could cause intermittent failures on certain hardware tokens. The `unsafe from_utf8_unchecked` should be replaced with safe alternatives. Overall architecture is sound with good RAII patterns and state management.
