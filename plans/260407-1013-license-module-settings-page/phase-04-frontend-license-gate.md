---
phase: 4
title: "Frontend LicenseGate + Blocking Screens"
status: done
effort: 1.5h
depends_on: [2]
completed: 2026-04-07
---

# Phase 4: Frontend LicenseGate + Blocking Screens

## Context Links
- [Plan](plan.md) | [Phase 2](phase-02-rust-tauri-commands.md)
- `src/App.tsx` — app root where LicenseGate wraps content
- `src/components/login-token-modal.tsx` — existing modal pattern
- `src/hooks/use-token-status.ts` — existing polling pattern

## Overview
- **Priority**: P1
- **Status**: pending
- **Description**: Create LicenseGate root wrapper that checks license on app launch. Routes to blocking screens (NoToken, NoLicense, Error) if verification fails. App content only renders on valid license.

## Key Insights
- LicenseGate is enforcement; Settings LicenseSection is informational. Completely separate concerns.
- NoTokenScreen polls `get_token_status` every 2s — reuse existing hook pattern
- All blocking screens are full-screen, no bypass, no skip, no cancel
- Loading state on mount — neutral spinner, no flash of content

## Requirements

### Functional
- On mount: call `check_license`, show loading spinner
- Route to correct screen based on result state
- `ok` → render children (normal app)
- `no_token` → NoTokenScreen (polls, auto-transitions when token detected)
- `no_license` → NoLicenseScreen (static, "contact IT")
- `error` → ErrorScreen (shows specific error message)

### Non-Functional
- No flash of app content before verification completes
- No cryptographic details, file paths, or error codes shown to user
- Clean, professional appearance matching app design system

## Architecture

```
src/components/license-gate.tsx         — root wrapper (~60 lines)
src/components/license-screens.tsx      — NoToken + NoLicense + Error screens (~100 lines)
```

Two files, not four: the screens are simple enough to colocate in one file. Each screen is a small function component (20-30 lines each).

## Related Code Files

### Files to Create
- `src/components/license-gate.tsx`
- `src/components/license-screens.tsx`

### Files to Modify
- `src/App.tsx` — wrap app content with `<LicenseGate>`
- `src/types/index.ts` — add LicenseCheckResult type
- `src/lib/tauri-api.ts` — add checkLicense API function

## Implementation Steps

### 1. Add types to `src/types/index.ts`
```typescript
export interface LicenseCheckResult {
  state: "ok" | "no_token" | "no_license" | "error";
  error_msg: string | null;
}
```

### 2. Add API function to `src/lib/tauri-api.ts`
```typescript
export const checkLicense = () =>
  invoke<LicenseCheckResult>("check_license");
```

### 3. Create `license-screens.tsx`

**NoTokenScreen:**
- Heading: "Token Required"
- Body: "Please insert your Bit4ID token to continue."
- Animated pulsing indicator (CSS animation)
- Polls `get_token_status` every 2000ms
- When token detected: calls `onTokenDetected` callback → LicenseGate re-checks license

**NoLicenseScreen:**
<!-- Updated: Validation Session 1 - Add Export + Import buttons for self-service -->
- Heading: "License Not Found"
- Body: "This application is not licensed for this machine. Please contact your IT department or use the buttons below."
- "Export Machine Credential" button — calls `exportMachineCredential()`, shows saved path
- "Import License" button — file picker for .dat, calls `importLicenseFile(path)`, triggers LicenseGate re-check on success

**ErrorScreen:**
- Heading varies by error type
- Body: user-facing error message from `check_license` result
- Error messages per spec:
  - MachineMismatch → "This license is not valid on this machine. Please contact IT."
  - Expired → "Your license has expired. Please contact IT for renewal."
  - TokenMismatch → "The inserted token does not match this machine license."
  - InvalidKey/Corrupted → "License file is invalid or has been tampered with. Please contact IT."

### 4. Create `license-gate.tsx`

<!-- Updated: Validation Session 1 - cfg(debug) bypass + NoLicenseScreen callbacks -->
```tsx
export default function LicenseGate({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<"loading" | "ok" | "no_token" | "no_license" | "error">("loading");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const runCheck = useCallback(async () => {
    try {
      const result = await checkLicense();
      setState(result.state);
      setErrorMsg(result.error_msg);
    } catch {
      setState("error");
      setErrorMsg("Failed to verify license. Please restart the application.");
    }
  }, []);

  useEffect(() => { runCheck(); }, [runCheck]);

  if (state === "loading") return <LoadingSpinner />;
  if (state === "no_token") return <NoTokenScreen onTokenDetected={runCheck} />;
  if (state === "no_license") return <NoLicenseScreen onLicenseImported={runCheck} />;
  if (state === "error") return <ErrorScreen errorMsg={errorMsg} />;
  return <>{children}</>;
}
```
Note: `cfg(debug_assertions)` bypass is in the Rust `check_license` command (Phase 2) — it returns `ok` in debug builds, so the frontend LicenseGate automatically passes without needing frontend-side dev logic.

### 5. Wrap app in `App.tsx`
```tsx
import LicenseGate from "./components/license-gate";

export default function App() {
  return (
    <LicenseGate>
      <AppProvider>
        <BrowserRouter>
          {/* ... existing app shell ... */}
        </BrowserRouter>
      </AppProvider>
    </LicenseGate>
  );
}
```

LicenseGate wraps OUTSIDE AppProvider — if license fails, no need to initialize app context, DB connections, etc.

## Todo List
- [x] Add LicenseCheckResult type to types/index.ts
- [x] Add checkLicense API function to tauri-api.ts
- [x] Create license-screens.tsx with 3 screen components
- [x] Create license-gate.tsx wrapper component
- [x] Wrap App content with LicenseGate in App.tsx
- [x] Verify compile and visual appearance

## Success Criteria
- App shows loading spinner on launch, then either app content or blocking screen
- NoTokenScreen polls and auto-transitions when token inserted
- No app content flashes before verification
- Error messages match spec exactly
- No bypass mechanism exists

## Risk Assessment
- **False blocking**: If license check has a bug, app is completely inaccessible. Mitigation: thorough testing of all LicenseState paths. Consider a dev-mode bypass flag (compile-time only, NOT runtime).
- **Polling performance**: NoTokenScreen polls every 2s. Low cost — `get_token_status` is a fast PKCS#11 call.

## Security Considerations
- LicenseGate wraps OUTSIDE AppProvider — no app state initialized if license fails
- No skip, cancel, or dismiss on any blocking screen
- Error messages are user-friendly, no technical details exposed
- Challenge-response in backend prevents binary patching bypass
