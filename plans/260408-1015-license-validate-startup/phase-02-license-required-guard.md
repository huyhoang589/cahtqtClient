# Phase 2: Create LicenseRequired Guard + LicenseNotFoundPage

## Overview
- **Priority:** P1
- **Status:** Complete
- **Description:** Create route guard component and the license-not-found page content.

## Key Insights
- Reuse existing styles from `license-screens.tsx` (screenStyle, cardStyle, etc.)
- Per spec: only Export Machine Credential button on blocked pages, no Import License
- Message: "License Not Found. This application is not licensed for this machine. Use the button below to Export Machine Credential then contact your admin department."
- Include note pointing to Settings for Import License

## Related Code Files
- **Create:** `src/components/license-required.tsx`
- **Create:** `src/components/license-not-found-page.tsx`
- **Read:** `src/components/license-screens.tsx` (reuse styles + export logic)
- **Read:** `src/contexts/app-context.tsx` (consume license state)

## Implementation Steps

1. Create `src/components/license-not-found-page.tsx`:
   - Full page content (not fullscreen — renders inside app-content area)
   - Centered card layout with:
     - Warning icon (reuse pulseStyle or simple SVG)
     - Heading: "License Not Found"
     - Body text per spec
     - `[Export Machine Credential]` button — calls `exportMachineCredential()` from tauri-api
     - Hint text: "To import a license file, go to Settings."
   - Feedback state for export result (success path / error)
   - Styles: adapt from license-screens.tsx but use `height: 100%` not `100vh` (renders inside layout)

2. Create `src/components/license-required.tsx`:
   ```typescript
   export default function LicenseRequired({ children }: { children: React.ReactNode }) {
     const { license } = useAppContext();
     
     if (license.licenseState === "loading") {
       return <LoadingSpinner />;  // simple centered spinner
     }
     
     if (license.licenseState !== "ok") {
       return <LicenseNotFoundPage />;
     }
     
     return <>{children}</>;
   }
   ```

## Todo List
- [x] Create `src/components/license-not-found-page.tsx`
- [x] Create `src/components/license-required.tsx`
- [x] Verify Export Machine Credential button works
- [x] Verify TypeScript compiles

## Success Criteria
- `<LicenseRequired>` renders children when licensed
- Shows LicenseNotFoundPage when unlicensed (any non-ok state)
- Export Machine Credential saves file and shows feedback
