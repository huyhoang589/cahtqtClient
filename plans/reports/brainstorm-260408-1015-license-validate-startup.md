# Brainstorm: License Validate at Startup v1

**Date:** 2026-04-08
**Branch:** feature/licenseValidateStartup

## Problem Statement

Current `LicenseGate` component wraps the entire app, blocking ALL pages (including Settings) when license is invalid. Users cannot access Settings to configure communication certificate or import license — creating a deadlock.

## Requirements (from spec)

1. App starts normally
2. If license not found or error:
   - Settings page fully accessible
   - Encrypt/Decrypt pages blocked with prompt
   - Prompt shows: "License Not Found. This application is not licensed for this machine. Use the button below to Export Machine Credential then contact your admin department"
   - Only Export Machine Credential button on blocked pages (Import License lives in Settings)
3. Settings page: info message "Communication certificate must configured for licensing"

## Evaluated Approaches

### A. Per-Page Route Guard (Selected)
Remove `LicenseGate` wrapper. Create `<LicenseRequired>` component wrapping only Encrypt/Decrypt/Partners routes.

**Pros:** Clean separation, Settings always accessible, minimal coupling, easy to extend
**Cons:** Need to move license state into AppContext (minor refactor)

### B. LicenseGate Pass-Through
Keep LicenseGate but change to context-only provider, push blocking into pages.

**Pros:** Less structural change
**Cons:** LicenseGate becomes misleading name, mixed responsibilities, more complex

## Final Design

### Architecture
```
App.tsx
├── AppProvider (license state + recheckLicense())
│   ├── /encrypt  → <LicenseRequired> → <EncryptPage>
│   ├── /decrypt  → <LicenseRequired> → <DecryptPage>
│   ├── /partners → <LicenseRequired> → <PartnersPage>
│   └── /settings → <SettingsPage> (always accessible)
```

### Component: LicenseRequired
- Route guard wrapping protected pages
- Reads license state from AppContext
- States: loading → spinner, ok → children, anything else → LicenseNotFoundPage

### Component: LicenseNotFoundPage
- Full page replacement (not modal)
- Warning icon + "License Not Found" heading
- Explanation text per spec
- Export Machine Credential button (reuses existing API)
- Note pointing to Settings for Import License

### LicenseSection Enhancement
- Info banner at top: "Communication certificate must be configured for licensing"
- Shown as guidance to direct user workflow

### Files to Modify
| File | Change |
|------|--------|
| `src/App.tsx` | Remove LicenseGate, add LicenseRequired to routes |
| `src/contexts/app-context.tsx` | Add license state + recheckLicense() |
| `src/components/license-gate.tsx` | Replace with license-required route guard |
| `src/components/license-screens.tsx` | Simplify to LicenseNotFoundPage |
| `src/pages/Settings/LicenseSection.tsx` | Add cert info banner |

### Risks
- Token polling: must preserve auto-recheck when token inserted
- Race condition: handle "loading" state during Rust startup check
- After license import in Settings, license context must update so guards allow access

### Success Criteria
- Settings accessible regardless of license state
- Encrypt/Decrypt/Partners show license prompt when unlicensed
- Export Machine Credential works from prompt page
- After importing license in Settings, protected pages become accessible
- Info banner visible in LicenseSection about comm cert requirement
