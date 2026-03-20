# CAHTQT System Architecture

**Version:** 1.0.0 + UI Change v6 (Initial Implementation + UI Design System Rebuild + eToken PKCS#11 Module + CryptoModule Change v5 + HTQT v2 FFI Redesign + UI Change v5 + UI Change v6 Visual Redesign)
**Last Updated:** 2026-03-12

## Architecture Overview

CAHTQT is a **3-tier desktop application** combining Tauri (thin IPC layer), Rust backend (business logic + FFI), and React frontend (UI).

```
┌─────────────────────────────────────────────────────────┐
│         React 18 Frontend (TypeScript)                  │
│  (Encrypt, Decrypt, Groups, Settings Pages)            │
└────────────────────┬────────────────────────────────────┘
                     │ Tauri IPC (async commands)
┌────────────────────▼────────────────────────────────────┐
│    Tauri v2 (Command Handler, State Manager)            │
│  (AppState: db pool, dll, token_login, operation_flag)  │
└────────────────────┬────────────────────────────────────┘
                     │ Rust Backend
        ┌────────────┴────────────┬──────────────┐
        │                         │              │
┌───────▼──────────┐  ┌──────────▼────┐  ┌──────▼────────┐
│ Database Layer   │  │ FFI Bridge     │  │ PKI Services  │
│ (SQLx + SQLite)  │  │ (libloading)   │  │ (cryptoki)    │
│                  │  │                │  │               │
│ - Settings       │  │ - htqt_crypto  │  │ - PKCS#11     │
│ - Partners       │  │   .dll (NEW)   │  │   token mgmt  │
│ - Partner Mbrs   │  │   encHTQT      │  │ - Cert parse  │
│ - Logs           │  │   decHTQT      │  │ (x509-parser) │
└──────────────────┘  └────────────────┘  └───────────────┘
        │                      │                    │
        │                      │                    │
     [SQLite]         [htqt_crypto.dll]    [Hardware Token]
    App Data Dir       Exe Directory          (Smart Card/HSM)
```

## Component Architecture

### 1. Frontend (React + TypeScript)

#### Design System (UPDATED in UI Change v6)
**File:** `src/styles.css` (40+ CSS design tokens)

**Color Palette (v3 Theme - UI Change v6):**
- **Accent:** `--color-accent-primary: #00c6e0` (brighter teal), `--color-accent-primary-dark: #009ab0` (hover state)
- **Background:** `--color-bg-content: #f0f4f8` (light content), `--color-bg-window: #1a1f2e` (window), `--color-bg-sidebar: #12161f` (deeper dark sidebar), `--color-bg-surface: #ffffff` (white panels), `--color-bg-log: #0d1117` (log panel)
- **Text:** `--color-text-on-light: #1e293b` (dark on light), `--color-text-on-light-2: #334155` (secondary), `--color-text-on-dark: #f1f5f9` (light on dark)
- **Status:** Red, yellow, green variants
- **Shadows:** `--shadow-card` (subtle), `--shadow-card-hover` (elevated on hover)
- **Border Radius:** sm 6px, md 10px, lg 14px, xl 18px (increased from v2)

**Fonts:**
- **UI:** `@fontsource/inter` (sans-serif, UI components)
- **Monospace:** `@fontsource/jetbrains-mono` (log panel, code display)

**Components:**
- Radix UI: Dialog (modals), Popover (tooltips/detail panels)
- Lucide React: Icon library (replacing emoji)
- CSS Classes: Component styling via design tokens (no Tailwind, no inline styles)

#### Page Structure
- **EncryptPage** (default route)
  - File selection
  - Group/recipient chooser
  - Progress monitor
  - Emits: `encrypt_batch` command
  - Listens: `encrypt_progress` events

- **DecryptPage**
  - Encrypted file selection
  - Progress monitor
  - PIN dialog (if token-based)
  - Emits: `decrypt_batch` command
  - Listens: `decrypt_progress` events

- **GroupsPage**
  - Create/list/delete groups
  - Add/remove recipients
  - Cert import & preview
  - Emits: group CRUD commands

- **SettingsPage**
  - Sender identity (certificate-based)
  - Integrated TokenSection for eToken/PKCS#11 scanning
  - Login/Logout buttons for token PIN management (NEW)
  - Emits: `set_setting`, `token_scan`, `token_export_sender_cert`, `login_token`, `logout_token` commands

#### Component Tree (3-Panel Layout — Updated UI Change v6)
```
App
├── AppHeader (56px top bar - white bg, gradient logo tile, stacked title, status pill)
├── AppSidebar (200px left nav - deep dark bg, gradient active items, dimmed inactive)
├── Main Container (flex: 1)
│   ├── Router
│   │   ├── /encrypt → EncryptPage (≤200 lines via useEncryptPanelResize hook)
│   │   │   ├── useSettingsStore() — output_data_dir
│   │   │   ├── useFileStatuses() — encrypt_progress events
│   │   │   ├── FileListPanel (with Status column)
│   │   │   ├── PartnerSelectPanel
│   │   │   ├── TokenWarningBar (shows token status)
│   │   │   ├── LoginTokenModal (orange PIN dialog)
│   │   │   ├── EncryptProgressPanel (sticky result header inside dark log box)
│   │   │   ├── "Open Folder" button (uses getAppSettings)
│   │   │   ├── Auto-reset on completion + duplicate file prompt
│   │   │   └── ConfirmEncryptDialog (Radix)
│   │   ├── /decrypt → DecryptPage
│   │   │   ├── useSettingsStore() — output_data_dir
│   │   │   ├── useFileStatuses() — decrypt_progress events
│   │   │   ├── FileListPanel (with Status column)
│   │   │   ├── PartnerSelectSimple
│   │   │   ├── TokenWarningBar (shows token status)
│   │   │   ├── LoginTokenModal (orange PIN dialog)
│   │   │   ├── DecryptProgressPanel
│   │   │   ├── "Open Folder" button (uses getAppSettings)
│   │   │   └── Auto-reset on completion + duplicate file prompt
│   │   ├── /partners → PartnersPage (renamed from GroupsPage)
│   │   │   ├── useSettingsStore() — output_data_dir
│   │   │   ├── invoke('get_app_settings') — desktop fallback
│   │   │   ├── GroupListSidebar
│   │   │   ├── RecipientTable
│   │   │   │   ├── PartnerMemberRow (extracted component)
│   │   │   │   └── MemberActionButtons (3 inline emoji buttons: 🔗📤×)
│   │   │   ├── PartnerDetailPanel (no "Certificate File" row)
│   │   │   ├── CreateGroupDialog (Radix)
│   │   │   ├── AddRecipientDialog (Radix)
│   │   │   ├── PinDialog (for Set Comm. action)
│   │   │   └── CertDetailPopover (Radix)
│   │   └── /settings → SettingsPage
│   │       ├── useSettingsStore() — output_data_dir, pkcs11_mode, pkcs11_manual_path
│   │       ├── OutputDataDirSection (text input + browse + clear)
│   │       ├── SenderCertIdentity
│   │       └── TokenSection
│   │           ├── LibraryStatus (display detected library info)
│   │           ├── ScanButton (trigger token_scan)
│   │           ├── TokenList (display slots + tokens)
│   │           ├── CertificateTable (display certs on selected token)
│   │           ├── CertificateDetail (show X.509 details)
│   │           └── LibraryPathInput (Auto/Manual radio mode)
│   └── LogPanel (140px bottom - aggregated Tauri events)
```

**Layout Grid:**
- Header: 56px height, full width
- Sidebar: 200px width, left
- Main: flex:1, center (max-width content area)
- Right Panel: 260px width, right
- Log Panel: 140px height, bottom of main (within flex:1 container)

### 2. Backend (Rust + Tauri)

#### Module Dependency Graph
```
lib.rs (AppState manager)
  ├── commands/mod.rs (command registration)
  │   ├── settings.rs (config commands: get_settings, set_setting, get_app_settings, open_folder)
  │   ├── partners.rs (partner CRUD + export_member_cert)
  │   ├── encrypt.rs (batch encryption with htqt_crypto.dll)
  │   ├── decrypt.rs (batch decryption with htqt_crypto.dll)
  │   ├── communication.rs (set_communication command — crypto op)
  │   ├── etoken.rs (token scanning, cert export, login/logout)
  │   └── logs.rs (operation history)
  ├── db/mod.rs (SQLite pool + migrations)
  │   ├── settings_repo.rs
  │   ├── partners_repo.rs
  │   ├── partner_members_repo.rs
  │   └── logs_repo.rs
  ├── htqt_ffi/ (FFI via libloading → htqt_crypto.dll — v2 callback architecture)
  │   ├── mod.rs
  │   ├── types.rs (CryptoCallbacksV2, FileEntry, RecipientEntry, BatchEncryptParams)
  │   ├── lib_loader.rs (HtqtLib with enc_multi, dec_v2)
  │   ├── token_context.rs (TokenContext + open_token_session)
  │   ├── callbacks.rs (5 unsafe C callbacks)
  │   └── error_codes.rs (error mapping + names)
  ├── cert_parser.rs (X.509 parsing)
  ├── etoken/ (PKCS#11 integration)
  │   ├── mod.rs (scanning runner)
  │   ├── models.rs (TokenScanResult, SlotInfo, TokenLoginState + sender_cert_path)
  │   ├── library_detector.rs (auto-detect eToken)
  │   ├── token_manager.rs (cryptoki ops)
  │   ├── certificate_reader.rs (read certs)
  │   └── certificate_exporter.rs (export sender cert)
  └── models.rs (shared data structures)
```

#### Command Execution Flow

**Encrypt Batch Example (NEW Flow):**
```
Frontend: useTokenStatus polling @ 20s, shows status dot
User clicks Encrypt → TokenWarningBar checks token_login state
     ↓
If Disconnected/Connected: Show LoginTokenModal
  User enters PIN → invoke('login_token', { pin })
     ↓
login_token cmd:
  1. Check token state (from etoken module)
  2. Store PIN in AppState.token_login = LoggedIn { pin }
  3. Emit token_status_changed event
     ↓
Frontend: useTokenStatus updates, LoginTokenModal closes
User selects files, partner → invoke('encrypt_batch', { partner_id, file_paths })
     ↓
Tauri IPC → Rust command handler (encrypt.rs)
     ↓
1. Load DLL from Arc<Mutex<Option<HtqtDll>>>
2. Fetch partner & members (N certificates) from DB
3. Retrieve cached PIN from AppState.token_login
4. Call: dll.encHTQT(file_paths, cert_datas, output_dir)
   - DLL outputs: DATA/ENCRYPT/{partner_name}/{stem}_{DDMMYYYY}.sf
   - Single DLL call (multi-cert, multi-file)
5. Emit: encrypt_progress event after call
6. Log operation to enc_logs table
7. Return result (file paths, summary)
     ↓
Frontend receives result, updates UI
```

**Decrypt Batch Example (NEW Flow):**
```
Similar to encrypt: LoginTokenModal checks token status → decrypt_batch calls dll.decHTQT()
Output: DATA/DECRYPT/{partner_name}/{stem} (no extension)
```

**Logout Flow:**
```
User clicks Settings → TokenSection → Logout button
  → invoke('logout_token')
     ↓
Backend: Zero PIN from AppState.token_login, set to Disconnected
         Emit token_status_changed event
     ↓
Frontend: useTokenStatus updates status dot, TokenWarningBar shows "Logged out"
```

### 3. Database Layer (SQLx + SQLite)

#### Database Location (UPDATED in UI Change v4)
- **Prior versions:** `{APPDATA}/CAHTQT/cahtqt.db` (root of app data)
- **Current version:** `{APPDATA}/CAHTQT/DATA/DB/cahtqt.db` (versioned subdirectory)
- **Auto-migration:** On first run, app detects old database location and migrates to new path (zero data loss)
- **Rationale:** Organizes output files + DB under `DATA/` hierarchy; enables future schema versioning

#### Schema

**settings** (Key-Value Store)
```sql
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT
);
```
Keys: `pkcs11_library_path`, `sender_cn`, `sender_org` (DLL path no longer stored; DLL auto-located)

**partners** (formerly `groups`)
```sql
CREATE TABLE partners (
  partner_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**partner_members** (formerly `recipients`)
```sql
CREATE TABLE partner_members (
  member_id TEXT PRIMARY KEY,
  partner_id TEXT NOT NULL,
  alias TEXT NOT NULL,
  cert_data BLOB NOT NULL,  -- DER-encoded X.509
  cert_org TEXT,            -- Organization from cert (NEW in migration 004)
  added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (partner_id) REFERENCES partners(partner_id)
);
```

**enc_logs**
```sql
CREATE TABLE enc_logs (
  operation_id TEXT PRIMARY KEY,
  batch_name TEXT,
  file_count INTEGER,
  member_count INTEGER,
  status TEXT,  -- 'Success', 'Failed', 'Partial'
  timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### Access Pattern
- **Settings:** Single-row KV lookups (fast)
- **Groups:** Filtered by user selection (small result sets)
- **Recipients:** Fetched by group_id on each encrypt/decrypt batch
- **Logs:** Paginated list with limit/offset

#### Migrations
All migrations run automatically on app startup via `db::init_db()`.

**Migration History:**
1. **001_init.sql** — Initial schema (settings, partners, partner_members, enc_logs)
2. **002_rename_tables.sql** — Renamed `groups` → `partners`, `recipients` → `partner_members`
3. **003_migrate_settings_keys.sql** — Renamed settings keys for eToken module
4. **004_add_cert_org.sql** — Added `cert_org TEXT` column to partner_members for organization metadata
   - Sets `cert_org = NULL` for existing rows (displays as "—" in UI until cert re-imported)
   - Runs on next app launch if `PRAGMA user_version < 4`
   - No data loss; backward-compatible schema change

**Path Migration (UI Change v4):**
- On app startup, checks if old `cahtqt.db` exists at root of `{APPDATA}/CAHTQT/`
- If found: moves to `{APPDATA}/CAHTQT/DATA/DB/cahtqt.db`, creates directory if needed
- If new path exists: uses existing database (no duplicate migration)
- All subsequent operations use new path
- Backward-compatible: old database not deleted (can be manually removed after verification)

**Current schema version:** 4.0.0 (PRAGMA user_version = 4)

**Migration Safety:**
- Each migration has fallback checks (IF NOT EXISTS, IF COLUMN EXISTS, etc.)
- Path migration handles both concurrent and sequential startup scenarios
- Rollback not supported; migrations are forward-only
- Legacy data retained in partner_members (cert_org nullable)

### 4. FFI Bridge (libloading) — htqt_crypto.dll

#### DLL Contract
**File:** `htqt_crypto.dll` (replaces `crypto_dll.dll`)
**Location:** App executable directory
**Features:** Multi-certificate support, PKCS#11 integrated, PIN not required per-operation, comprehensive error codes

**Exported C Functions:**
```c
int encHTQT(
  const char* file_paths[],      // Paths to files to encrypt
  int file_count,                // Number of files
  const unsigned char* cert_datas[], // DER-encoded certs (concatenated)
  const int* cert_lengths,       // Length of each cert
  int cert_count,                // Number of recipients
  const char* output_dir,        // Where to save encrypted files
  char* error_msg_out            // Error message buffer (caller allocates)
);

int decHTQT(
  const char* file_paths[],      // Encrypted file paths (.sf extension assumed)
  int file_count,
  const char* output_dir,        // Where to save decrypted files
  char* error_msg_out
);

const char* HTQT_GetError(void); // Get last error message (thread-safe)

// (NEW in v5) Diagnostics
const char* HTQT_GetPkcs11Lib(void); // Returns eToken library path
unsigned int HTQT_GetTokenSlotID(void); // Returns eToken slot ID
```

**Key Differences from v0:**
- PIN no longer passed per-operation; managed via `login_token` command
- Supports multiple certificates per call (N recipients in single operation)
- Output files automatically named: `{stem}_{DDMMYYYY}.sf` (encrypt), `{stem}` (decrypt)
- Error retrieval via separate function call
- Error codes mapped to human-readable names (27 codes: -25 to -1)
- Diagnostic functions for troubleshooting PKCS#11 library/slot mismatches

**Error Code Mapping (v5):**
- `-25` to `-1`: Specific error codes (PKCS#11 init failure, cert not found, PIN invalid, etc.)
- `0`: Success
- `1+`: Reserved for future use

**Wrapper Class:** `HtqtLib` (Rust)
```rust
impl HtqtLib {
  fn load(path: &str) -> Result<Self>  // Resolves 3 symbols: encHTQT_multi, decHTQT_v2, HTQT_GetError
  fn enc_multi(&self, params: &BatchEncryptParams, cbs: &CryptoCallbacksV2, ctx: &TokenContext, results: &mut [BatchResult]) -> Result<i32, String>
  fn dec_v2(&self, sf_path: &str, out_path: &str, recipient_id: &str, cbs: &CryptoCallbacksV2, ctx: &TokenContext) -> Result<(), String>
  fn get_error() -> String
}
```

**Callback Type Aliases:**
- `FnRsaPssSign` — Sign digest with token's private key via cryptoki
- `FnRsaOaepEncCert` — Encrypt plaintext with cert's public key (software RSA via `rsa` crate)
- `FnRsaOaepDecrypt` — Decrypt ciphertext with token's private key via cryptoki
- `FnRsaPssVerify` — Verify signature with sender cert's public key (software RSA via `rsa` crate)
- `FnProgress` — Emit "encrypt-progress" Tauri event

**Callback Implementations (in callbacks.rs):**
- **cb_rsa_pss_sign** — Unsafe FFI; casts user_ctx to TokenContext, calls session.sign() via cryptoki
- **cb_rsa_oaep_enc_cert** — Unsafe FFI; parses cert_der via x509-parser, encrypts via `rsa` crate
- **cb_rsa_oaep_decrypt** — Unsafe FFI; calls session.decrypt() via cryptoki
- **cb_rsa_pss_verify** — Unsafe FFI; parses sender_cert_der, verifies via `rsa` crate
- **cb_progress** — Unsafe FFI; emits "encrypt-progress" event (decrypt does not use)

**TokenContext Integration:**
- `open_token_session(pkcs11: &Pkcs11, slot, pin, app, own_cert_der, event_name) -> TokenContext`
- Holds PKCS#11 session (RW mode, user-logged-in)
- Holds AppHandle for emitting progress events
- On Drop: session logout + close
- Reuses Pkcs11 instance from AppState.etoken module (no re-initialize)

**Command Integration (v2):**
- `encrypt.rs`: Opens TokenContext → builds FileEntry[], RecipientEntry[] → calls lib.enc_multi() inside spawn_blocking
- `decrypt.rs`: Opens TokenContext → per-file loop: calls lib.dec_v2() inside spawn_blocking with recipient_id from AppState.token_login.cert_cn
- Both pass `&TokenContext` as user_ctx to DLL callbacks
- No DllDiagnostic logging (PKCS#11 ownership moved to Rust etoken module)

### 5. eToken/PKI Services (NEW)

#### eToken PKCS#11 Integration Module
**Location:** `src-tauri/src/etoken/`
**Purpose:** Detect, initialize, and read certificates from eToken hardware tokens via PKCS#11

**Module Structure:**
- **library_detector.rs** → Auto-detect eToken middleware from standard Windows paths (bit4ID Universal Middleware registry/common locations)
- **token_manager.rs** → Initialize cryptoki, enumerate slots, manage sessions
- **certificate_reader.rs** → Read X.509 certificates from PKCS#11 token objects
- **certificate_exporter.rs** → Extract sender identity (CN, ORG) from exported certificate

**Commands (in `src-tauri/src/commands/etoken.rs`):**
1. `token_scan(lib_path_override?)` → async, spawns blocking task, returns `TokenScanResult`
   - Auto-detects library OR uses override path
   - Enumerates all slots + tokens
   - Reads all X.509 certs from each token
   - Caches raw_der in `AppState.last_token_scan` (not sent to frontend)
   - Result: `{ library_info, tokens: [{ slot_id, slot_label, tokens: [{ label, serial, certs: [{ label, subject, issuer, ... }] }] }] }`

2. `token_get_library_info()` → returns detected library info (vendor, version, path)

3. `token_export_sender_cert(cert_label?, pin?)` → runs on blocking thread, exports from cache
   - Uses `last_token_scan.raw_der` to find cert
   - Extracts CN (common name) + ORG from X.509
   - Returns `{ cn, org, cert_pem }`

4. `token_set_library_path(path: String)` → saves to `settings.pkcs11_library_path`

5. `token_clear_sender_cert()` → clears cached sender cert from `AppState`

**Commands (in `src-tauri/src/commands/settings.rs` — Enhanced UI Change v2):**
6. `get_app_settings()` → returns `{ output_data_dir, pkcs11_mode, pkcs11_manual_path }`
   - Reads 3 settings from KV store
   - Falls back `output_data_dir` to `USERPROFILE/Desktop` if empty

7. `open_folder(path: String)` → creates directory if needed, opens in Windows Explorer
   - Validates path; prevents directory traversal

8. `export_member_cert(cert_path: String, dest_dir: String)` → copies member cert file to destination
   - Creates dest_dir if needed
   - Returns destination file path on success

**Commands (in `src-tauri/src/commands/communication.rs` — NEW):**
9. `set_communication(recipient_cert_path, partner_name, dest_dir, pin)` → encrypts sender cert to recipient
   - Validates token logged in + sender_cert_path available
   - Opens PKCS#11 session with provided PIN
   - Calls `encHTQT_multi` with 1 source file (sender cert) + 1 recipient cert
   - Output: `{dest_dir}/SetComm_{partner_name}_{DDMMYYYY}.sf`
   - Returns output file path on success; emits app_log

**Frontend Integration (TokenSection):**
- **LibraryStatus.tsx** → Shows detected library path + vendor
- **ScanButton.tsx** → Calls `token_scan()`, handles loading/error states
- **TokenList.tsx** → Lists slots + tokens from scan result
- **CertificateTable.tsx** → Shows certs on selected token (label, subject, issuer, validity)
- **CertificateDetail.tsx** → Detailed cert view + "Set as Sender" button
- **useTokenScan.ts** → React hook managing token scan state (loading, error, data)

#### X.509 Parsing (x509-parser 0.16)
- **Purpose:** Validate & display cert metadata
- **Data Extracted:**
  - Subject Distinguished Name (extract CN for sender identity)
  - Issuer DN
  - Validity (not before / not after)
  - Key usage flags
  - Serial number
  - Organization (ORG extraction via certificate_exporter)

**Used in:**
- Token certificate list display (TokenSection)
- Cert detail popover (recipient table)
- Expiry badge (visual warnings)

## State Management

### Shared State (AppState)
Managed by Tauri and accessible to all commands:
```rust
pub struct AppState {
  pub db: SqlitePool,                              // DB connection pool
  pub dll: Arc<Mutex<Option<HtqtDll>>>,           // Loaded htqt_crypto.dll (if available)
  pub is_operation_running: Arc<Mutex<bool>>,     // Pauses PKCS#11 polling
  pub last_token_scan: Arc<Mutex<Option<TokenScanResult>>>,  // Cache token scan result with raw_der
  pub token_login: Arc<Mutex<TokenLoginState>>,   // (NEW) Token connection + login state + cached PIN
}
```

**TokenLoginState (Enhanced in UI Change v2):**
```rust
pub enum TokenLoginState {
  Disconnected,           // No token detected
  Connected,              // Token found, not logged in
  LoggedIn {
    pin: Vec<u8>,        // Cached PIN (zeroized on logout/disconnect)
    timestamp: SystemTime,
    sender_cert_path: Option<String>,  // Path to saved sender.crt (NEW)
  },
}
```

### Frontend State (React Hooks)
- **Component State:** useState for UI toggles, dialogs, selections
- **Async State:** No centralized state manager (simple command-based IPC)
- **Cache:** None (fresh DB fetches on each action)

### Real-Time Updates (Tauri Emitter)
```rust
// Backend emits
app.emit("encrypt_progress", EncryptProgressPayload { ... })?;
app.emit("app_log", AppLogPayload { timestamp, level, message, context })?;

// Frontend listens
const unlisten = await listen<EncryptProgressPayload>('encrypt_progress', (event) => {
  setProgress(event.payload);
});
const unlistenLog = await listen<AppLogPayload>('app_log', (event) => {
  setLogs(prev => [...prev, event.payload]);
});
```

Events emitted:
- `encrypt_progress` → frequency: M × N calls (file/recipient progress)
- `decrypt_progress` → frequency: M calls (file-level progress)
- `app_log` → frequency: per operation (NEW - aggregated in LogPanel)

## Error Handling

### Frontend
- Try-catch on all Tauri invocations
- User-friendly toast/dialog messages
- Graceful degradation (DLL optional)

### Backend
- Custom error types per module (`dll_error.rs`)
- Error propagation via `?` operator
- All DB operations return `Result<T>`
- DLL not found → logged, continues without encryption

### FFI
- C function return codes (0 = success, -1 = error)
- Error messages passed via out buffer
- PIN cleared from memory on any path

## Security Considerations

### 1. PIN Handling (CHANGED)
- Collected via `LoginTokenModal` component (orange Radix dialog)
- Passed to Rust via `login_token` command (in-memory string)
- Stored in AppState.token_login after cryptoki session established
- Zeroized on explicit `logout_token` command OR automatic disconnect
- PIN NOT passed per-operation; reused for all encHTQT/decHTQT calls during session
- Never logged or persisted to disk

### 2. Certificate Storage
- Stored as raw DER-encoded bytes in DB (no secrets)
- X.509 validation on import
- Expiry checked on display (not enforced)

### 3. DLL Trust
- `htqt_crypto.dll` loaded from exe directory only (not system path)
- Optional (app works without it, graceful degradation)
- No code signing check (responsibility of deployment)
- Old `crypto_dll.dll` FFI completely replaced

### 4. Build Hardening
- Release: LTO, single codegen unit, stripped symbols
- Panic: `abort` (no unwinding)
- No debug symbols in final binary

## Performance Considerations

### Encryption Throughput
- **Bottleneck:** DLL (C) execution time
- **M×N Scaling:** Linear with operations count
- **Progress Events:** One per DLL call (minimal overhead)

### Database Performance
- **Pool Size:** Default (4 connections)
- **Schema:** No indexes (small tables expected)
- **Queries:** Simple, all execute < 10ms

### Memory
- **Startup:** ~50MB (Tauri runtime + Rust libs)
- **Operating:** ~80MB (steady state)
- **Large Batches:** Streaming to DLL (no full file buffer)

### UI Responsiveness
- **Async Runtime:** tokio (non-blocking)
- **Long Operations:** Emitter + progress panel (responsive)
- **Startup Time:** ~500ms (DB init + optional DLL load)

## Deployment Architecture

### Windows NSIS Installer
- Generated by Tauri (automated)
- Single executable + bundled Rust libraries
- Installs to Program Files
- Start menu entry
- Auto-update capable (not currently configured)

### Runtime Dependencies
- Windows 7+ (NSIS default)
- Visual C++ runtime (bundled by Tauri)
- Optional: PKCS#11 library (.dll) at runtime

### Configuration Files
- SQLite DB: `%APPDATA%/CAHTQT/` (Tauri app data dir)
- DLL: Loaded from app exe directory (optional)
- PKCS#11 lib: Path specified in settings

## Extension Points

### Adding a New Encryption Format
1. Modify `models.rs` → add `EncryptFormat` variant
2. Update DLL contract → add new export function
3. Add UI selector in `EncryptPage`
4. Update encrypt command handler

### Adding a New Page
1. Create `pages/NewPage.tsx`
2. Add route to `App.tsx`
3. Add sidebar link in `app-sidebar.tsx`
4. Implement page component + subcomponents

### Adding Database Tables
1. Add migration SQL to `db/mod.rs`
2. Create new repository module `db/new_repo.rs`
3. Register in `db/mod.rs`
4. Create commands in `commands/` as needed

## Monitoring & Diagnostics

### Logging
- **Tauri:** Automatically logs to console in debug build
- **Rust:** No centralized logging (can add `tracing` crate)
- **Frontend:** Browser console (F12) in dev mode

### Status Indicators (UI)
- **StatusBar:** DLL loaded? PKCS#11 available?
- **Settings:** Test DLL path, test PKCS#11 library
- **Cert Expiry:** Badge color indicates time to expiry

### Operation Logs
- **Table:** `enc_logs` tracks all encrypt/decrypt batches
- **Access:** `LogsPage` (reads from `list_logs` command)
- **Retention:** No automatic cleanup (manual delete via DB)

## Scalability Limits

| Aspect | Limit | Notes |
|--------|-------|-------|
| Group Count | 1000+ | No pagination (list_groups unbounded) |
| Recipients/Group | 100+ | M×N operations linear |
| Files/Batch | 1000+ | DLL handles per-call |
| DLL Size | 10MB max | Memory impact on load |
| DB Size | 1GB+ | SQLite handles large files |

Practical limits depend on:
- File sizes (no streaming implemented)
- DLL performance (external)
- PKCS#11 token speed (hardware-dependent)
