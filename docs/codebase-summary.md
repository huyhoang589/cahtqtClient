# CAHTQT Codebase Summary

**Project:** CAHTQT PKI Encryption Desktop App
**Version:** 1.0.0-crypto-v1
**Status:** Initial Implementation + UI Redesign v1 + eToken PKCS#11 Module + CryptoModule Change v1
**Last Updated:** 2026-03-06

## Overview

CAHTQT is a production-ready desktop application for M×N encryption using PKI cryptography. Built with Tauri v2 (Rust) and React 18 (TypeScript), it provides a user-friendly interface for encrypting files for multiple recipients and decrypting received encrypted files.

**Key Capability:** Encrypt M files for N recipients in a single batch → M×N DLL operations with real-time progress tracking.

## Architecture Layers

### Backend (Rust/Tauri)
**Location:** `src-tauri/src/`

#### Core Components

| Module | Purpose |
|--------|---------|
| `lib.rs` | Tauri app bootstrapper, AppState management, command registration |
| `main.rs` | Binary entry point |
| `models.rs` | Data structures (Settings, Partner, PartnerMember, LogEntry, TokenLoginState) |
| `htqt_ffi.rs` | FFI bridge to htqt_crypto.dll (encHTQT/decHTQT C exports) |
| `cert_parser.rs` | X.509 certificate parsing via x509-parser 0.16 |
| `etoken/` | PKCS#11 token integration, auto-detection, certificate reading/export |
| `app_log.rs` | Application event logging helper |

#### Database Layer (`src-tauri/src/db/`)

| Module | Purpose |
|--------|---------|
| `mod.rs` | SQLite pool init, migrations (001_init.sql, 002_rename_tables.sql, 003_migrate_settings_keys.sql) |
| `settings_repo.rs` | App settings CRUD (DLL path, PKCS#11 library path, sender cert path, sender CN) |
| `partners_repo.rs` | Partner (formerly "Group") management |
| `partner_members_repo.rs` | PartnerMember (formerly "Recipient") storage per partner |
| `logs_repo.rs` | Encryption/decryption operation logs |

**Tables (Post-Migration 003):**
- `settings` (key-value store) — keys include `pkcs11_library_path` (renamed from `pkcs11_lib_path`)
- `partners` (partner_id, name, created_at) — renamed from `groups`
- `partner_members` (member_id, partner_id, alias, cert_data, added_at) — renamed from `recipients`
- `enc_logs` (operation_id, batch_name, file_count, member_count, status, timestamp) — field update: recipient_count → member_count

#### Commands Layer (`src-tauri/src/commands/`)

**Settings Commands:**
- `get_settings()` → app config + DLL status
- `set_setting(key, value)` → staged write to DB
- `get_app_info()` → version, app data dir
- `is_dll_loaded()` → bool

**Token & Login Commands (NEW in CryptoModule v1):**
- `login_token(pin)` → Result (authenticate to token, store PIN in AppState, emit token_status_changed)
- `logout_token()` → Result (zeroize PIN, reset token state)
- `get_token_status()` → TokenStatus (Disconnected | Connected | LoggedIn)
- `token_scan(lib_path_override?)` → TokenScanResult (slots, tokens, certificates)
- `token_get_library_info()` → LibraryInfo (vendor, version, loaded path)
- `token_export_sender_cert(cert_label?, pin?)` → SenderCertExportResult (with CN, ORG extraction)
- `token_set_library_path(path)` → Result (save PKCS#11 library path)
- `token_clear_sender_cert()` → Result (clear cached sender cert)

**Partner Commands:**
- `create_partner(name)` → PartnerId
- `list_partners()` → Vec<Partner>
- `rename_partner(partner_id, new_name)` → Result
- `delete_partner(partner_id)` → Result
- `add_partner_member(partner_id, alias, cert_data)` → MemberId
- `list_partner_members(partner_id)` → Vec<PartnerMember>
- `delete_partner_member(member_id)` → Result
- `import_cert_preview(pem_bytes)` → CertInfo (subject, issuer, issuer_cn, org, file_path, expiry)
- `import_sender_cert(pem_bytes)` → Result (NEW: for SenderCertIdentity)

**Encrypt/Decrypt Commands (NEW: use htqt_crypto.dll):**
- `encrypt_batch(partner_id, file_paths, file_format)` → emits progress events, outputs to DATA/ENCRYPT/{partner_name}/{stem}_{DDMMYYYY}.sf
- `decrypt_batch(file_paths)` → emits progress events, outputs to DATA/DECRYPT/{partner_name}/{stem}
- `list_logs(limit, offset)` → Vec<LogEntry>

**Event Emission (Tauri Emitter):**
- `encrypt_progress` → {current_file, total_files, current_recipient, total_recipients}
- `decrypt_progress` → {current_file, total_files}
- `app_log` → {timestamp, level, message, context} (aggregated in log panel)
- `token_status_changed` → TokenStatus (NEW: emitted on login/logout/scan)

### Frontend (React/TypeScript)
**Location:** `src/`

#### Pages
| File | Purpose |
|------|---------|
| `EncryptPage.tsx` | Select files, choose partner, resizable split layout (files LEFT, partner select RIGHT) |
| `DecryptPage.tsx` | Partner list LEFT, encrypted file list RIGHT, auto-resolves to DATA/DECRYPT/{partner}/ |
| `GroupsPage.tsx` | Create/rename/delete partners, manage members per partner, light sidebar style |
| `SettingsPage.tsx` | Sender cert identity, integrated TokenSection with Login/Logout buttons (no DLL path config) |
| `Settings/TokenSection/` | eToken PKCS#11 integration: library auto-detect, slot scan, token enumeration, cert list, export, login/logout |

#### Core Components

| Component | Purpose |
|-----------|---------|
| `App.tsx` | Router setup, 3-panel layout (Header + Sidebar + Main + LogPanel, no right panel) |
| `app-header.tsx` | Top header bar (56px) with title, version, status indicators |
| `app-sidebar.tsx` | Navigation sidebar (200px, dark bg) — labels updated to Partners |
| `log-panel.tsx` | Bottom log panel (140px, resizable, min 80px) aggregating app events |
| `file-list-panel.tsx` | File selection UI (drag-drop, file picker) |
| `partner-select-simple.tsx` | Partner selection dropdown for encryption |
| `partner-select-panel.tsx` | Multi-partner selection for encryption batch |
| `encrypt-progress-panel.tsx` | Real-time progress bar (M×N grid) |
| `decrypt-progress-panel.tsx` | Real-time progress bar for decryption |
| `group-list-sidebar.tsx` | Partner list for PartnersPage (light style) |
| `partner-detail-panel.tsx` | Partner detail panel with member list |
| `create-group-dialog.tsx` | Partner creation (Radix Dialog) |
| `add-recipient-dialog.tsx` | Add member to partner (Radix Dialog) |
| `recipient-table.tsx` | Display members with cert info, expiry badge |
| `cert-detail-popover.tsx` | Certificate details (Radix Popover) |
| `cert-expiry-badge.tsx` | Visual warning (red/yellow) for expiring certs |
| `confirm-encrypt-dialog.tsx` | M×N summary before encrypt (Radix Dialog) |
| `login-token-modal.tsx` | (NEW) Orange Radix Dialog for PIN entry, shown on encrypt/decrypt if token not logged in |
| `token-warning-bar.tsx` | (NEW) Status bar showing token connection state (Disconnected/Connected/LoggedIn) |
| `sender-cert-identity.tsx` | Configure sender certificate identity (cert-based) |
| `TokenSection/` | PKCS#11 token integration UI (auto-detect, scan, cert list, export, login/logout buttons) |

#### Routing (React Router v6)
```
/encrypt   → EncryptPage (files LEFT, partner select RIGHT, resizable divider)
/decrypt   → DecryptPage (partner list LEFT, file list RIGHT)
/partners  → PartnersPage (partner list sidebar, detail panel)
/settings  → SettingsPage (sender cert, DLL, PKCS#11 config)
/          → redirect to /encrypt
```

## Technology Stack

### Backend
- **Tauri:** v2 (desktop framework, IPC bridge)
- **Rust:** 2021 edition
- **Database:** SQLx 0.8 + SQLite (async runtime: tokio)
- **FFI:** libloading 0.8 (dynamic DLL loading)
- **X.509:** x509-parser 0.16 (cert parsing)
- **PKCS#11:** cryptoki 0.6 (token enumeration)
- **Security:** zeroize 1.8 (sensitive data cleanup)
- **Serialization:** serde 1.0, serde_json 1.0
- **Async Runtime:** tokio 1.0 (full features)

### Frontend
- **React:** 18.3.1
- **TypeScript:** 5.5.3
- **Routing:** react-router-dom 6.26.0
- **Build Tool:** Vite 5.4.2
- **Tauri API:** @tauri-apps/api v2
- **Dialog Plugin:** @tauri-apps/plugin-dialog v2
- **Shell Plugin:** @tauri-apps/plugin-shell v2
- **UI Primitives:** @radix-ui/react-dialog, @radix-ui/react-popover
- **Icons:** lucide-react
- **Fonts:** @fontsource/inter, @fontsource/jetbrains-mono

### Desktop Packaging
- **Windows Installer:** NSIS (auto-configured by Tauri)
- **Executable:** Fully self-contained, runs from Program Files

## Data Flow (NEW in CryptoModule v1)

### Encryption Flow
1. User on EncryptPage; `TokenWarningBar` shows token status (via 20s polling)
2. If token not LoggedIn: LoginTokenModal appears → user enters PIN
3. Frontend calls `login_token(pin)` → backend stores PIN in AppState.token_login
4. User selects M files in LEFT panel, partner (N members) in RIGHT panel
5. Frontend calls `encrypt_batch(partner_id, file_paths, format)`
6. Backend loads htqt_crypto.dll, calls `encHTQT(file_paths, cert_datas, output_dir)` (single call)
7. DLL reads cached PIN from AppState.token_login, performs encryption
8. DLL outputs: DATA/ENCRYPT/{partner_name}/{stem}_{DDMMYYYY}.sf
9. Progress emitted after call
10. Results logged to `enc_logs` table
11. Resizable divider ratio persisted via `setSetting`

### Decryption Flow
1. Similar to encrypt: TokenWarningBar + LoginTokenModal if needed
2. User selects partner from LEFT list (DecryptPage)
3. User selects encrypted .sf files from RIGHT list
4. Frontend calls `decrypt_batch(file_paths)`
5. Backend loads htqt_crypto.dll, calls `decHTQT(file_paths, output_dir)` (single call)
6. DLL reads cached PIN, performs decryption
7. DLL outputs: DATA/DECRYPT/{partner_name}/{stem} (no extension)
8. Progress emitted
9. Results logged to `enc_logs`

### Logout Flow
1. User clicks Settings → TokenSection → Logout button
2. Frontend calls `logout_token()`
3. Backend zeros PIN, sets token_login = Disconnected, emits token_status_changed
4. TokenWarningBar updates status dot

### Partner Management Flow
1. User creates partner (PartnersPage → `create_partner`)
2. User adds members (→ import cert via `import_cert_preview`, then `add_partner_member`)
3. Partner + members stored in SQLite (tables: partners, partner_members)
4. Encryption operations reference partner_id
5. Decryption auto-resolves to partner_name directory

## Security Highlights

- **PIN Zeroization:** zeroize 1.8 clears PIN from memory after token access
- **FFI Safety:** libloading loads DLL only when available (graceful degradation)
- **X.509 Validation:** x509-parser validates cert structure and expiry
- **Async I/O:** tokio prevents blocking during long encrypt/decrypt ops
- **Build Hardening:** Release build uses LTO, 1 codegen unit, strip symbols

## Database Schema (Post-Migration 002)

### settings
```
key TEXT PRIMARY KEY
value TEXT
```
**Keys:** pkcs11_library_path, sender_cn, sender_org (DLL path no longer stored; htqt_crypto.dll auto-located from exe directory)

### partners
```
partner_id TEXT PRIMARY KEY
name TEXT
created_at TIMESTAMP
```
(Renamed from `groups` via migration 002)

### partner_members
```
member_id TEXT PRIMARY KEY
partner_id TEXT FOREIGN KEY
alias TEXT
cert_data BLOB (DER-encoded X.509)
added_at TIMESTAMP
```
(Renamed from `recipients` via migration 002)

### enc_logs
```
operation_id TEXT PRIMARY KEY
batch_name TEXT
file_count INTEGER
member_count INTEGER (renamed from recipient_count)
status TEXT (Success, Failed, Partial)
timestamp TIMESTAMP
```

## Build & Deployment

### Development
```bash
cd src-tauri
cargo build

cd ..
npm install
npm run dev
```

### Release Build
```bash
npm run build
npm run tauri build
```

Creates Windows installer at `src-tauri/target/release/bundle/nsis/`.

## File Structure
```
F:/.PROJECT/.CAHTQT.PROJ/
├── src-tauri/                      # Rust/Tauri backend
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   ├── models.rs                # Partner, PartnerMember, CertInfo, TokenLoginState
│   │   ├── htqt_ffi.rs              # FFI bridge to htqt_crypto.dll (encHTQT, decHTQT, HTQT_GetError)
│   │   ├── cert_parser.rs           # X.509 certificate parsing
│   │   ├── app_log.rs               # App logging helper
│   │   ├── etoken/                  # PKCS#11 token integration
│   │   │   ├── mod.rs               # Module exports, token_scan runner
│   │   │   ├── models.rs            # TokenScanResult, SlotInfo, TokenInfo, etc.
│   │   │   ├── library_detector.rs  # Auto-detect eToken middleware
│   │   │   ├── token_manager.rs     # cryptoki initialization, slot enumeration
│   │   │   ├── certificate_reader.rs # Read certs from token
│   │   │   └── certificate_exporter.rs # Export sender cert (CN, ORG extraction)
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── settings.rs
│   │   │   ├── partners.rs
│   │   │   ├── encrypt.rs           # Uses htqt_crypto.dll.encHTQT (NEW)
│   │   │   ├── decrypt.rs           # Uses htqt_crypto.dll.decHTQT (NEW)
│   │   │   ├── etoken.rs            # token_scan, login_token, logout_token, get_token_status (NEW)
│   │   │   └── logs.rs
│   │   └── db/
│   │       ├── mod.rs               # Database initialization, migrations
│   │       ├── settings_repo.rs
│   │       ├── partners_repo.rs
│   │       ├── partner_members_repo.rs
│   │       └── logs_repo.rs
│   ├── Cargo.toml
│   └── build.rs
├── src/                            # React/TypeScript frontend
│   ├── main.tsx
│   ├── App.tsx                      # Layout: Header | Sidebar | Main (files/partner list) | Log Panel
│   ├── styles.css                  # 40+ CSS design tokens
│   ├── pages/
│   │   ├── EncryptPage.tsx         # Resizable split: LEFT (files), RIGHT (partner select)
│   │   ├── DecryptPage.tsx         # Side-by-side: LEFT (partner list), RIGHT (files)
│   │   ├── GroupsPage.tsx          # Partner list sidebar, detail panel
│   │   ├── SettingsPage.tsx        # Sender cert, integrated TokenSection (no DLL config)
│   │   └── Settings/TokenSection/  # eToken PKCS#11 integration UI
│   │       ├── TokenSection.tsx    # Main component (library status, scan, login/logout buttons)
│   │       ├── LibraryStatus.tsx   # Display library info + loaded path
│   │       ├── ScanButton.tsx      # Trigger token_scan command
│   │       ├── TokenList.tsx       # List detected tokens/slots
│   │       ├── CertificateTable.tsx # List certificates on selected token
│   │       ├── CertificateDetail.tsx # Show cert details (subject, issuer, validity)
│   │       └── useTokenScan.ts     # React hook for token scanning (20s polling)
│   ├── components/                 # 20+ reusable components (Radix + CSS tokens)
│   └── hooks/
│       ├── use-encrypt.ts          # Encrypt state management
│       ├── use-decrypt.ts          # Decrypt state management
│       └── use-log-panel.ts        # Log panel event aggregation
├── package.json
├── tsconfig.json
├── vite.config.ts
└── docs/                           # Documentation
```

## Performance Characteristics

- **M×N Encryption:** Scales linearly with file count × recipient count
- **Real-time Progress:** Emitter events sent after each DLL call (minimal overhead)
- **SQLite:** Single connection pool (async) for all DB ops
- **Startup:** ~500ms (DB init + optional DLL load)

## Known Limitations & Future Enhancements

- Single file format output (configurable via settings)
- PKCS#11 token polling every N seconds (can be fine-tuned)
- No built-in key backup/restore (PIN-locked in token only)
- Manual recipient management (no LDAP/directory integration)

## Support & Maintenance

All core functionality is production-ready. Refer to:
- `./docs/system-architecture.md` for detailed design
- `./docs/code-standards.md` for development guidelines
- `./docs/development-roadmap.md` for planned features
