---
phase: 3
title: "Frontend License Section in Settings"
status: done
effort: 1.5h
depends_on: [2]
completed: 2026-04-07
---

# Phase 3: Frontend License Section in Settings

## Context Links
- [Plan](plan.md) | [Phase 2](phase-02-rust-tauri-commands.md)
- `src/pages/Settings/CommunicationSection.tsx` — pattern to follow
- `src/pages/SettingsPage.tsx` — where to add section
- `src/types/index.ts` — type definitions
- `src/lib/tauri-api.ts` — Tauri invoke wrappers

## Overview
- **Priority**: P1
- **Status**: pending
- **Description**: Create LicenseSection component for Settings page. Shows license status badge + expiry. Buttons for credential export and license import.

## Key Insights
- Follow CommunicationSection pattern: `useEffect` on mount to fetch data, local state, inline styles with CSS vars
- Status badge uses existing CSS var color system (success/warning/error)
- Export saves to output_data_dir (no file picker needed — just a button)
- Import uses existing `selectFiles` pattern from tauri-api.ts

## Requirements

### Functional
- Display license status badge with color coding
- Display expiry date (only when status=valid), "Perpetual" if null
- "Export Machine Credential" button — saves JSON to output_data_dir, shows success message
- "Import License" button — file picker for .dat files, validates + imports, refreshes display

### Non-Functional
- No machine fingerprint, token serial, or crypto details shown in UI
- Follow existing styling patterns (CSS custom properties, inline styles)
- Component < 200 lines (considering modularization if needed)

## Architecture

```
src/pages/Settings/LicenseSection.tsx  — main component (~120 lines)
```

Single component is sufficient — simpler than TokenSection which has multiple sub-components because of its complexity. LicenseSection has minimal UI.

## Related Code Files

### Files to Create
- `src/pages/Settings/LicenseSection.tsx`

### Files to Modify
- `src/types/index.ts` — add license types
- `src/lib/tauri-api.ts` — add license API functions
- `src/pages/SettingsPage.tsx` — import + render LicenseSection

## Implementation Steps

### 1. Add types to `src/types/index.ts`
```typescript
// ---- License Module Types -----------------------------------------------------

export type LicenseStatus =
  | "valid" | "expired" | "not_found" | "no_token"
  | "token_mismatch" | "machine_mismatch" | "corrupted";

export interface LicenseInfo {
  status: LicenseStatus;
  expires_at: number | null;  // unix timestamp, null = perpetual
  product: string | null;
}

export interface MachineCredentialResult {
  saved_path: string;
  token_serial: string;
  user_name: string;
}

export interface ImportLicenseResult {
  status: LicenseStatus;
  expires_at: number | null;
}
```

### 2. Add API functions to `src/lib/tauri-api.ts`
```typescript
// ---- License -----------------------------------------------------------------

export const getLicenseInfo = () =>
  invoke<LicenseInfo>("get_license_info");

export const exportMachineCredential = () =>
  invoke<MachineCredentialResult>("export_machine_credential");

export const importLicenseFile = (filePath: string) =>
  invoke<ImportLicenseResult>("import_license_file", { filePath });
```
Add `LicenseInfo`, `MachineCredentialResult`, `ImportLicenseResult` to the import block.

### 3. Create `LicenseSection.tsx`

**Structure:**
```
LICENSE section title + description
├── Status card (always visible)
│   ├── Status badge (colored dot + text)
│   └── Expiry line (only when valid)
├── Action buttons row
│   ├── "Export Machine Credential" button
│   └── "Import License" button
└── Success/error feedback (temporary message)
```

**Status badge color mapping:**
| Status | Color var | Label |
|--------|-----------|-------|
| valid | --cahtqt-color-success | Valid |
| expired | --cahtqt-color-error | Expired |
| not_found | --cahtqt-text-muted | Not Found |
| no_token | --cahtqt-color-warning | No Token |
| token_mismatch | --cahtqt-color-error | Token Mismatch |
| machine_mismatch | --cahtqt-color-error | Machine Mismatch |
| corrupted | --cahtqt-color-error | Corrupted |

**Component logic:**
- `useEffect` on mount: call `getLicenseInfo()`, store in state
- Export button: call `exportMachineCredential()`, show saved_path in success message
- Import button: call `selectFiles([{ name: "License", extensions: ["dat"] }])`, then `importLicenseFile(path)`, refresh license info
- Feedback: use `useState` for message, auto-clear after 3s with `setTimeout`

### 4. Add to SettingsPage.tsx
After CommunicationSection, add divider + LicenseSection:
```tsx
<div style={{ borderBottom: "1px solid var(--cahtqt-border-light)", marginBottom: 24, marginTop: 24 }} />
<LicenseSection />
```

## Todo List
- [x] Add license types to types/index.ts
- [x] Add license API functions to tauri-api.ts
- [x] Create LicenseSection.tsx component
- [x] Add LicenseSection to SettingsPage.tsx
- [x] Verify compile with `npm run build` or dev server

## Success Criteria
- LicenseSection renders in Settings page below Communication
- Status badge shows correct color + label for each status
- Export button saves credential JSON and shows success path
- Import button opens file picker, imports, and refreshes status
- No machine fingerprint or token serial visible in UI

## Risk Assessment
- **Token not inserted during export**: Export will fail. Show clear error message "Please insert your Bit4ID token".
- **Import of wrong file type**: Backend validates; frontend pre-filters to .dat extension.

## Security Considerations
- No sensitive data displayed (no fingerprint, no serial, no paths in status card)
- Export is deliberate user action — user consciously saves credential file
- Import validates via backend before persisting — frontend just passes file path
