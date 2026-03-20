# CAHTQT Development Roadmap

**Last Updated:** 2026-03-12
**Current Release:** v1.0.0 + UI Change v6 (Major Visual Redesign + Encrypt Error Fix - Complete)

## Executive Summary

CAHTQT PKI Encryption Desktop App has completed all 8 implementation phases, UI design system rebuild, and major UI Change v6 redesign. v1.0.0 + visual overhaul includes light content areas, deep sidebar, gradient header logo, and critical DLL error handling fix. All core features fully implemented and tested.

## Release Timeline

### Phase 1: Foundation & Architecture ✓ COMPLETE
**Status:** Complete
**Dates:** Planning Phase (completed pre-development)

**Deliverables:**
- Tauri v2 project setup (Windows/NSIS target)
- React 18 + TypeScript + Vite configuration
- Rust backend skeleton (lib.rs, main.rs)
- SQLite database schema design
- Architecture documentation

**Acceptance Criteria:** ✓ All met
- Project builds without errors
- Tauri dev server runs
- Frontend dev server runs
- Database schema created

---

### Phase 2: Core Backend Infrastructure ✓ COMPLETE
**Status:** Complete
**Dates:** Development (completed)

**Deliverables:**
- `db/` module with settings, groups, recipients, logs repositories
- SQLx async database initialization + migrations
- `models.rs` - data structures (Group, Recipient, Settings, LogEntry)
- Tauri state management (AppState with db pool, dll, operation_running)
- Command handler registration

**Acceptance Criteria:** ✓ All met
- All CRUD operations work (tested manually)
- Database migrations run on startup
- No SQL errors or panics
- AppState shared to all commands

---

### Phase 3: PKI & PKCS#11 Integration ✓ COMPLETE
**Status:** Complete
**Dates:** Development (completed)

**Deliverables:**
- `cert_parser.rs` - X.509 parsing via x509-parser 0.16
  - Subject, issuer, validity extraction
  - Certificate validation
- `pkcs11_service.rs` - PKCS#11 token enumeration via cryptoki 0.6
  - Token discovery
  - Certificate listing
  - PIN collection (zeroized)
- `commands/settings.rs` - PKCS#11 commands
  - `scan_token_certs()` - enumerate available certificates
  - `get_settings()` - read app configuration
  - `set_setting()` - update settings (DLL path, PKCS#11 lib, etc.)

**Acceptance Criteria:** ✓ All met
- Certificates parsed without errors
- PKCS#11 library loaded successfully
- Certificate details displayed correctly
- PIN zeroized after use

---

### Phase 4: DLL FFI Bridge ✓ COMPLETE
**Status:** Complete
**Dates:** Development (completed)

**Deliverables:**
- `dll_wrapper.rs` - libloading 0.8 wrapper
  - Dynamic DLL loading (crypto_dll.dll)
  - C extern function definitions (EncryptFiles, DecryptFiles)
  - Error handling (DllError type)
  - PIN zeroization (zeroize 1.8)
- DLL optional (graceful degradation if not found)
- `dll_error.rs` - Custom error types

**Acceptance Criteria:** ✓ All met
- DLL loaded from exe directory
- C functions callable via safe Rust wrapper
- Error messages propagated to frontend
- App runs without DLL (encryption disabled)

---

### Phase 5: Encryption & Decryption Commands ✓ COMPLETE
**Status:** Complete
**Dates:** Development (completed)

**Deliverables:**
- `commands/encrypt.rs` - batch encryption
  - `encrypt_batch()` - M files × N recipients → M×N DLL calls
  - Real-time progress events (Tauri Emitter)
  - Logging to enc_logs table
- `commands/decrypt.rs` - batch decryption
  - `decrypt_batch()` - decrypt files, PIN required
  - Progress events
  - Logging
- `commands/logs.rs` - operation history
  - `list_logs()` - paginated log retrieval

**Acceptance Criteria:** ✓ All met
- M×N operations execute successfully
- Progress events emitted during batch
- All operations logged with metadata
- PIN collected and zeroized
- Error handling on DLL failure

---

### Phase 6: Recipient Group Management ✓ COMPLETE
**Status:** Complete
**Dates:** Development (completed)

**Deliverables:**
- `commands/groups.rs` - group CRUD
  - `create_group(name)` - new group creation
  - `list_groups()` - retrieve all groups
  - `rename_group()` - update group name
  - `delete_group()` - remove group
  - `add_recipient()` - add cert to group
  - `list_recipients()` - fetch group members
  - `delete_recipient()` - remove recipient
  - `import_cert_preview()` - validate + preview cert before adding

**Acceptance Criteria:** ✓ All met
- All group operations CRUD work
- Certificates imported and validated
- Recipients linked to groups correctly
- No orphaned records in database

---

### Phase 7: React Frontend UI ✓ COMPLETE
**Status:** Complete
**Dates:** Development (completed)

**Deliverables:**
- **Pages (4 total):**
  - `EncryptPage.tsx` - File selection, group/recipient chooser, progress
  - `DecryptPage.tsx` - Encrypted file selection, progress, PIN dialog
  - `GroupsPage.tsx` - Group management, recipient table, cert import
  - `SettingsPage.tsx` - DLL config, PKCS#11 config, sender identity, output dir

- **Components (18 total):**
  - Navigation: `app-sidebar.tsx`, `status-bar.tsx`, `group-list-sidebar.tsx`
  - File Handling: `file-list-panel.tsx`, `output-dir-picker.tsx`
  - Recipient Management: `recipient-select-panel.tsx`, `recipient-table.tsx`, `add-recipient-dialog.tsx`, `create-group-dialog.tsx`
  - Progress: `encrypt-progress-panel.tsx`, `decrypt-progress-panel.tsx`
  - Configuration: `dll-path-config.tsx`, `pkcs11-config.tsx`, `sender-identity-form.tsx`, `token-cert-list.tsx`
  - Dialogs: `confirm-encrypt-dialog.tsx`, `pin-dialog.tsx`, `cert-detail-popover.tsx`, `cert-expiry-badge.tsx`

- **Routing:** React Router v6 with 4 main routes + sidebar nav

**Acceptance Criteria:** ✓ All met
- All pages render without errors
- React Router navigation works
- Form inputs bind correctly
- Tauri commands invoked successfully
- Progress events received and displayed
- PIN dialog secure input works
- Certificates displayed with details

---

### Phase 7.5: UI Design System Rebuild ✓ COMPLETE
**Status:** Complete
**Dates:** Development (completed 2026-02-21)

**Deliverables:**
- **Design Token System** (`src/styles.css`)
  - 40+ CSS variables (colors, fonts, spacing, shadows)
  - Cyan accent (#00b4d8), light/dark/log backgrounds
  - Inter font (UI) + JetBrains Mono (log panel)

- **New Layout** (3-panel shell)
  - Header: 56px (app-header.tsx with status indicators)
  - Sidebar: 200px (dark bg #1a2340)
  - Main: flex:1 (light bg #e8f4fd)
  - Right Panel: 260px (page-specific summaries)
  - Log Panel: 140px (bottom, aggregated events)

- **New Components**
  - `app-header.tsx` - Top bar with title, version
  - `log-panel.tsx` - Event aggregator (encrypt/decrypt/app_log)
  - `right-panel.tsx` - Dispatcher to page-specific panels
  - `right-panel-encrypt-summary.tsx` - Encrypt stats
  - `right-panel-decrypt-status.tsx` - Decrypt status
  - `right-panel-group-stats.tsx` - Groups statistics
  - `right-panel-settings-config.tsx` - Settings status

- **New Hook**
  - `use-log-panel.ts` - Aggregate Tauri events

- **Radix UI Migration**
  - pin-dialog, confirm-encrypt-dialog, create-group-dialog, add-recipient-dialog (Dialog)
  - cert-detail-popover (Popover)

- **Icon Replacement**
  - lucide-react replacing emoji icons

- **State Lifting**
  - useEncrypt() moved to App.tsx, passed as props

- **Rust Backend**
  - `app_log.rs` module for logging
  - `AppLogPayload` struct in models.rs
  - New `app_log` Tauri event emission with timestamp

- **Deleted Components**
  - `status-bar.tsx` (merged into app-header + right-panel)

**Acceptance Criteria:** ✓ All met
- npm run build → success (4.24s)
- cargo check → success (zero errors)
- All components styled via CSS tokens (no Tailwind, no inline)
- Radix Dialog/Popover functioning correctly
- Log panel aggregates all event types
- New layout responsive and accessible

---

### Phase 8: Build & Packaging ✓ COMPLETE
**Status:** Complete
**Dates:** Development (completed)

**Deliverables:**
- Windows NSIS installer generation (Tauri automated)
- Release build configuration
  - LTO enabled, 1 codegen unit
  - Panic behavior set to abort
  - Symbols stripped
  - Binary optimized for size
- Build verification (no errors, all tests pass)
- Installer tested on Windows 10

**Acceptance Criteria:** ✓ All met
- Release build compiles cleanly
- NSIS installer generated
- Installer runs on Windows 10+
- App launches and functions correctly

---

### Phase 9: UI Change v6 — Visual Redesign + Encrypt Error Fix ✓ COMPLETE
**Status:** Complete
**Dates:** Development (completed 2026-03-12)

**Deliverables:**
- **CSS Token Overhaul** (`src/styles.css`)
  - Full color palette migration: light content, deep sidebar, white dialogs
  - Background: window #1a1f2e, sidebar #12161f, content #f0f4f8, surface #ffffff
  - Accent primary: #00b4d8 → #00c6e0 (brighter), added dark variant #009ab0
  - Border radius increased (sm 4→6, md 6→10, lg 8→14, xl 12→18)
  - New shadow tokens for card elevation effects
  - Progress bar: pill-shaped, light background, gradient fill
  - Input focus: 3px spread, new accent color
  - Scrollbars: light theme + dark variant for log panels

- **App Header Redesign** (`src/components/app-header.tsx`)
  - White background (#ffffff) with subtle shadow
  - 32×32 teal gradient logo tile (new branding)
  - Two-line stacked app name (CAHTQT PKI / PKI Encryption)
  - Token status wrapped in conditional green pill

- **App Sidebar Redesign** (`src/components/app-sidebar.tsx`)
  - Background: deeper dark (#12161f)
  - Subtle borders: rgba(255,255,255,0.05)
  - Active nav item: gradient + cyan glow
  - Inactive nav items: 45% opacity
  - Collapse toggle: JS-driven hover state

- **Encrypt Error Intercept** (`src-tauri/src/commands/encrypt.rs`)
  - DLL-level errors now emit progress events (status: "error")
  - Per-file error surface in progress panel (not app log)
  - Added `emit_dll_error_as_progress` helper function
  - Returns `Ok(EncryptResult)` instead of propagating Err

**Acceptance Criteria:** ✓ All met
- cargo check passes clean
- npm run build passes clean
- DLL error -33 surfaces as progress panel errors
- App log not polluted with DLL-level failures
- Header/Sidebar visual redesign matches v3 TechSpec
- All design tokens applied consistently
- Sidebar collapse toggle hover works correctly

---

## Current Status (v1.0.0 + UI Change v6)

### Completed Features
- ✓ M×N encryption with real-time progress
- ✓ Decryption of encrypted files
- ✓ Recipient group management
- ✓ X.509 certificate import & validation
- ✓ PKCS#11 token enumeration
- ✓ DLL FFI integration (optional)
- ✓ SQLite database with 4 tables
- ✓ Settings management (DLL path, PKCS#11, output dir, etc.)
- ✓ Operation logging + app event logging
- ✓ React UI with 4 pages + 25 components
- ✓ 3-panel layout (header, sidebar, main, right panel, log panel)
- ✓ Design token system (40+ CSS variables)
- ✓ Radix UI primitives (Dialog, Popover)
- ✓ Lucide React icons
- ✓ Windows NSIS installer
- ✓ UI Change v6 — Visual redesign (light content, deep sidebar, new header/sidebar) (NEW)
- ✓ Encrypt DLL error intercept (errors surface in progress panel, not app log) (NEW)

### Known Limitations (v1.0.0)
- Single output format (V1) - can extend in v1.1
- No LDAP/directory integration (manual recipient entry)
- No automatic key backup (PIN-locked in token only)
- PKCS#11 polling every N seconds (configurable)
- No key rotation (design limitation, out of scope)
- No built-in audit logs (operation logs basic, no compliance hooks)

### Metrics (v1.0.0 + UI Redesign)
| Metric | Value |
|--------|-------|
| Rust Code Lines | ~2,600 |
| TypeScript Lines | ~2,100 |
| CSS Design Tokens | 40+ |
| Database Tables | 4 |
| Tauri Commands | 17 |
| Tauri Events | 3 (encrypt_progress, decrypt_progress, app_log) |
| React Pages | 4 |
| React Components | 25 |
| Build Size (Release) | ~25 MB |
| Startup Time | ~500 ms |
| Installer Size | ~18 MB |

---

## Future Roadmap (v1.1+)

### v1.1 - Enhanced Encryption (Q2 2026)
**Priority:** Medium

**Planned Features:**
- [ ] Additional encryption formats (V2, XTS)
- [ ] Batch recipient import (CSV upload)
- [ ] Recipient expiry warnings (7 days before)
- [ ] Enhanced progress UI (file-by-file details)
- [ ] Search/filter in groups and recipients

**Estimated Effort:** 2-3 weeks

---

### v1.2 - Audit & Compliance (Q3 2026)
**Priority:** Medium

**Planned Features:**
- [ ] Extended operation logging (detailed metadata)
- [ ] Log export (CSV, JSON)
- [ ] User audit trail (who created/deleted group, when)
- [ ] Compliance report generation
- [ ] Log retention policies (auto-delete after N days)

**Estimated Effort:** 3-4 weeks

---

### v1.3 - Key Management (Q4 2026)
**Priority:** Low

**Planned Features:**
- [ ] Key rotation support (re-encrypt with new cert)
- [ ] Certificate lifecycle hooks (warnings, auto-disable expired)
- [ ] Backup/restore encrypted configurations
- [ ] Multi-token support

**Estimated Effort:** 4-5 weeks

---

### v2.0 - Enterprise Features (2027)
**Priority:** TBD

**Potential Features:**
- [ ] LDAP/Active Directory integration
- [ ] Web-based recipient management portal
- [ ] REST API for 3rd-party integration
- [ ] Hardware HSM support (beyond smart cards)
- [ ] Multi-user sessions with role-based access

**Estimated Effort:** 8+ weeks

---

## Success Metrics

### v1.0.0 (Current)
- [x] Build compiles without errors
- [x] All 8 phases complete
- [x] NSIS installer functional
- [x] All core features working
- [x] No critical bugs reported

### v1.1+
- [ ] User adoption threshold (100+ installations)
- [ ] Performance: Encrypt 1000 files × 100 recipients in < 5 minutes
- [ ] Uptime: App runs 24/7 without crashes
- [ ] User feedback: 90%+ satisfaction on core features

---

## Dependencies & Blockers

### External Dependencies
- **crypto_dll.dll** - Provided by client (required for encryption)
- **PKCS#11 Library** - Client provides (optional, for token support)
- **Windows 7+** - Runtime platform requirement

### Internal Dependencies
- None (v1.0.0 standalone)

### Known Blockers
- None currently blocking v1.1 work

---

## Maintenance Plan

### Current Release (v1.0.0)
- **Bug Fixes:** As reported
- **Security Patches:** Within 48 hours of disclosure
- **Updates:** Quarterly (March, June, September, December)

### Release Cycle
1. Feature development → PR review → merge to main
2. Tag release (e.g., v1.1.0)
3. Update CHANGELOG.md + roadmap
4. Build + test NSIS installer
5. Publish release (GitHub, internal distribution)

### Version Numbering
- **Major (X.0.0):** Breaking changes, major features
- **Minor (X.Y.0):** New features, backwards compatible
- **Patch (X.Y.Z):** Bug fixes only

---

## Review & Update Cadence

- **Monthly:** Check blockers, update progress
- **Quarterly:** Review roadmap alignment, adjust timelines
- **Annually:** Strategic assessment, plan next year
- **Per Release:** Update CHANGELOG + roadmap immediately

**Next Roadmap Review:** 2026-05-20
