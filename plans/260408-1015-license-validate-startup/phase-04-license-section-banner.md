# Phase 4: Add Cert Info Banner to LicenseSection

## Overview
- **Priority:** P2
- **Status:** Complete
- **Description:** Add **conditional** informational banner in LicenseSection — only shown when communication certificate is not yet configured.
<!-- Updated: Validation Session 1 - Banner changed from static to conditional (show only when no cert) -->

## Related Code Files
- **Modify:** `src/pages/Settings/LicenseSection.tsx`

## Implementation Steps

1. Check if comm cert is configured (via Tauri command or existing settings state)
2. **Conditionally** render info banner only when cert is NOT configured:
   ```tsx
   {!hasCommunicationCert && (
     <div style={{
       padding: "8px 12px",
       borderRadius: "var(--cahtqt-radius-md)",
       border: "1px solid var(--cahtqt-color-info, #3b82f6)",
       background: "rgba(59,130,246,0.08)",
       fontSize: "var(--cahtqt-font-size-sm)",
       color: "var(--cahtqt-text-on-light)",
     }}>
       ℹ Communication certificate must be configured before importing a license file.
     </div>
   )}
   ```
3. Place between the subtitle text and the status card
4. Determine cert status from existing settings/context or add a check via `getTokenStatus()` / similar API

## Todo List
- [x] Add info banner to LicenseSection.tsx
- [x] Verify banner displays correctly
- [x] Compile check

## Success Criteria
- Info banner visible in LicenseSection
- Message clearly guides user to configure comm cert first
- Consistent with existing design tokens/styles
