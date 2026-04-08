# Phase 1: Add License State to AppContext

## Overview
- **Priority:** P1
- **Status:** Complete
- **Description:** Extend AppContext with license state + recheckLicense() so route guards and any component can read license status reactively.

## Key Insights
- Current `LicenseGate` manages its own state via `useState` + `checkLicense()` call
- AppContext currently only has `tokenStatus` and `settings`
- Need to replicate LicenseGate's check logic into a reusable hook

## Related Code Files
- **Modify:** `src/contexts/app-context.tsx`
- **Create:** `src/hooks/use-license-status.ts` (new hook)
- **Read:** `src/lib/tauri-api.ts` (checkLicense API)
- **Read:** `src/types/index.ts` (LicenseCheckResult type)

## Implementation Steps

1. Create `src/hooks/use-license-status.ts`:
   ```typescript
   // Hook that calls checkLicense() on mount, exposes state + recheck
   export function useLicenseStatus() {
     const [state, setState] = useState<LicenseGateState>("loading");
     const [errorMsg, setErrorMsg] = useState<string | null>(null);
     
     const recheckLicense = useCallback(async () => {
       setState("loading");
       try {
         const result = await checkLicense();
         setState(result.state as LicenseGateState);
         setErrorMsg(result.error_msg);
       } catch {
         setState("error");
         setErrorMsg("Failed to verify license.");
       }
     }, []);
     
     useEffect(() => { recheckLicense(); }, [recheckLicense]);
     
     return { licenseState: state, licenseErrorMsg: errorMsg, recheckLicense };
   }
   ```
   - Type: `LicenseGateState = "loading" | "ok" | "no_token" | "no_license" | "error"`

2. Update `src/contexts/app-context.tsx`:
   - Import `useLicenseStatus`
   - Add to `AppContextValue`: `license: ReturnType<typeof useLicenseStatus>`
   - Call hook in `AppProvider`, pass into context value

## Todo List
- [x] Create `src/hooks/use-license-status.ts`
- [x] Update `src/contexts/app-context.tsx` to include license state
- [x] Verify TypeScript compiles with `npm run tauri dev`

## Success Criteria
- `useAppContext().license.licenseState` returns current license state
- `useAppContext().license.recheckLicense()` triggers re-verification
- No breaking changes to existing tokenStatus/settings consumers
