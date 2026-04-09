# Phase 2: Settings — SET KEY / REMOVE KEY

## Context
- [Phase 1](./phase-01-comm-key-service.md) — comm_key_service dependency
- [communication.rs](../../src-tauri/src/commands/communication.rs) — current comm cert commands
- [CommunicationSection.tsx](../../src/pages/Settings/CommunicationSection.tsx) — current UI

## Overview
- **Priority:** High
- **Status:** Complete
- **Description:** Replace direct cert browse/save with .sf1 communication key workflow: browse .sf1 → PIN prompt → decrypt → extract cert info → save to DB → copy .sf1 to COMM_KEY dir → delete temp cert.

## Key Insights
- Current `save_communication_cert()` takes a plain cert path, parses, copies to partners dir, saves DB setting
- New flow: browse .sf1 → decrypt with token → get cert → parse → save info → copy .sf1
- `remove_communication_key` must do full reset: delete .sf1 + clear DB + delete PartnerMember
- Frontend needs PIN prompt UI — check if existing login_token dialog can be reused
- Button state: SET KEY enabled ↔ REMOVE KEY disabled (mutually exclusive)

## Architecture

### Backend: New Tauri Commands
<!-- Updated: Validation Session 1 - Split into preview + confirm commands -->
```rust
// Step 1: Preview — decrypt and return cert info WITHOUT saving
#[tauri::command]
pub async fn preview_communication_key(
    sf1_path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CommunicationCertInfo, String>
// Flow:
// 1. Read token login state (pkcs11_lib, slot, PIN)
// 2. decrypt_comm_key(sf1_path, temp_dir) → temp_cert_path
// 3. cert_parser::parse_cert_file(temp_cert_path) → CertInfo
// 4. Store temp_cert_path in state (for cleanup if user cancels)
// 5. Return CommunicationCertInfo (preview only, NOT saved)

// Step 2: Confirm — save .sf1 + DB + cleanup temp cert
#[tauri::command]
pub async fn confirm_set_communication_key(
    sf1_path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<CommunicationCertInfo, String>
// Flow:
// 1. Copy .sf1 to DATA/COMM_KEY/comm_key.sf1
// 2. Save communication_cert_path = COMM_KEY path in settings DB
// 3. Save recipient info as PartnerMember (or update existing)
// 4. cleanup_temp_cert(temp_cert_path from preview)
// 5. Emit communication-cert-changed event
// 6. Return CommunicationCertInfo

// Cancel: cleanup temp cert without saving
#[tauri::command]
pub async fn cancel_preview_communication_key(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String>
// Flow: cleanup temp_cert_path from state

#[tauri::command]
pub async fn remove_communication_key(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String>
// Flow:
// 1. Delete .sf1 from DATA/COMM_KEY/
// 2. Clear communication_cert_path setting
// 3. Delete associated PartnerMember record
// 4. Emit communication-cert-changed event
```

### Frontend: CommunicationSection.tsx Changes
<!-- Updated: Validation Session 1 - Two-step preview UX flow -->
- Change "Browse Certificate..." to "Browse communication key..."
- File filter: `.sf1` files only
- **Two-step SET KEY flow:**
  1. Browse .sf1 → call `preview_communication_key(sf1_path)` → decrypt → return cert info
  2. Show cert info preview card (CN, serial, expiry, issuer) with "Confirm SET KEY" + "Cancel" buttons
  3. User confirms → call `confirm_set_communication_key(sf1_path)` → save .sf1 + DB
  4. User cancels → cleanup temp cert, return to Not Set state
- Replace "Clear" with "REMOVE KEY" button
- Add status indicator: Valid / Invalid / Not Set
- REMOVE KEY: calls `remove_communication_key()`

### Communication Key Status
```
Not Set: no .sf1 in COMM_KEY dir, no DB setting
Valid: .sf1 exists in COMM_KEY + recipient info in DB
Invalid: .sf1 exists but decrypt failed / cert info missing
```

## Related Code Files
- **Modify:** `src-tauri/src/commands/communication.rs` — replace `save_communication_cert` with `set_communication_key`, replace `clear_communication_cert` with `remove_communication_key`
- **Modify:** `src/pages/Settings/CommunicationSection.tsx` — new UI flow
- **Modify:** `src/lib/tauri-api.ts` — update API bindings
- **Modify:** `src-tauri/src/lib.rs` — update invoke_handler registrations
- **Modify:** `src/types.ts` — update types if needed

## Implementation Steps

### Backend
<!-- Updated: Validation Session 1 - Split into preview + confirm steps -->
1. In `communication.rs`, add `preview_communication_key` command:
   - Validate token is logged in (read from `state.token_login`)
   - Call `comm_key_service::decrypt_comm_key()` in spawn_blocking
   - Parse decrypted cert with `cert_parser::parse_cert_file()`
   - If parse fails → return error "Invalid communication key" + cleanup temp
   - Store temp_cert_path in AppState (e.g., `pending_comm_key_preview: Mutex<Option<String>>`)
   - Return `CommunicationCertInfo` for preview display

1b. Add `confirm_set_communication_key` command:
   - Copy .sf1 to `DATA/COMM_KEY/comm_key.sf1` (create dir, overwrite if exists)
   - Save `communication_cert_path` setting = path to .sf1 in COMM_KEY dir
   - Save recipient info to DB (PartnerMember or settings key-values)
   - Cleanup temp cert (from pending preview state)
   - Clear `pending_comm_key_preview` state
   - Emit `communication-cert-changed` event
   - Return `CommunicationCertInfo`

1c. Add `cancel_preview_communication_key` command:
   - Read + clear `pending_comm_key_preview` state
   - Cleanup temp cert file

2. In `communication.rs`, add `remove_communication_key` command:
   - Read `communication_cert_path` from settings
   - Delete .sf1 file from COMM_KEY dir
   - Clear `communication_cert_path` setting (set to "")
   - Clear recipient info from DB (delete PartnerMember with matching serial, or clear settings)
   - Emit `communication-cert-changed` event

3. Update `get_communication_cert` to work with new flow:
   - Instead of reading cert file directly, read saved recipient info from DB
   - No longer needs cert file on disk to return info (info saved at SET KEY time)

4. Remove or deprecate `save_communication_cert` (replaced by `set_communication_key`)

5. Update `lib.rs` invoke_handler: replace old commands with new ones

### Frontend
6. Update `CommunicationSection.tsx`:
   <!-- Updated: Validation Session 1 - Two-step preview flow -->
   - Browse now selects `.sf1` files (update file filter)
   - Change label to "Browse communication key..."
   - After browse: call `preview_communication_key(path)` → show loading spinner
   - On preview success: display cert info preview card (CN, serial, expiry, issuer)
   - Show "Confirm SET KEY" + "Cancel" buttons below preview
   - Confirm → call `confirm_set_communication_key(path)` → show "Valid" status
   - Cancel → call `cancel_preview_communication_key()` → return to Not Set
   - On preview error: show error message with status "Invalid"
   - REMOVE KEY button: visible when status is Valid

7. Update `tauri-api.ts`:
   - Add `previewCommunicationKey(sf1Path: string)`
   - Add `confirmSetCommunicationKey(sf1Path: string)`
   - Add `cancelPreviewCommunicationKey()`
   - Add `removeCommunicationKey()`
   - Keep `getCommunicationCert()` (reads saved info)
   - Remove `saveCommunicationCert()`

## Todo
- [x] Backend: `set_communication_key` command
- [x] Backend: `remove_communication_key` command
- [x] Backend: update `get_communication_cert` for new flow
- [x] Backend: remove deprecated `save_communication_cert`
- [x] Backend: update invoke_handler in `lib.rs`
- [x] Frontend: update CommunicationSection.tsx
- [x] Frontend: update tauri-api.ts bindings
- [x] Compile check

## Success Criteria
- Browse .sf1 → SET KEY → decrypt → cert info displayed → .sf1 stored in COMM_KEY
- REMOVE KEY → all traces cleared (file + DB + settings)
- Button states toggle correctly
- Status indicator shows Valid/Invalid/Not Set
- communication-cert-changed event emitted on both SET and REMOVE

## Risk Assessment
- Token not logged in when SET KEY clicked → clear error message, guide to login first
- .sf1 file not encrypted for this user's token → DLL returns error, show "Invalid key"
- Large .sf1 file → timeout concern → use spawn_blocking (already planned)

## Security Considerations
- PIN only accessed from token_login state (already Zeroizing<String>)
- Temp cert deleted immediately after parsing — never persists
- .sf1 in COMM_KEY is encrypted at rest — only decrypted when needed
