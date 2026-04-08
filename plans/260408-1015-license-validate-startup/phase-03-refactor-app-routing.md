# Phase 3: Refactor App.tsx Routing

## Overview
- **Priority:** P1
- **Status:** Complete
- **Description:** Remove LicenseGate wrapper, wrap protected routes with LicenseRequired.

## Related Code Files
- **Modify:** `src/App.tsx`

## Implementation Steps

1. Remove `LicenseGate` import and wrapper from App.tsx
2. Import `LicenseRequired` from `./components/license-required`
3. Wrap protected routes:
   ```tsx
   <Routes>
     <Route path="/" element={<Navigate to="/encrypt" replace />} />
     <Route path="/encrypt" element={<LicenseRequired><EncryptPage /></LicenseRequired>} />
     <Route path="/decrypt" element={<LicenseRequired><DecryptPage /></LicenseRequired>} />
     <Route path="/partners" element={<LicenseRequired><PartnersPage /></LicenseRequired>} />
     <Route path="/settings" element={<SettingsPage />} />
   </Routes>
   ```
4. Ensure `AppProvider` still wraps `BrowserRouter` (license context needed before routes render)

## Before → After

**Before:**
```
<LicenseGate>        ← blocks everything
  <AppProvider>
    <BrowserRouter>
      <Routes>...</Routes>
    </BrowserRouter>
  </AppProvider>
</LicenseGate>
```

**After:**
```
<AppProvider>         ← license state in context
  <BrowserRouter>
    <Routes>
      /encrypt  → <LicenseRequired>...</LicenseRequired>
      /decrypt  → <LicenseRequired>...</LicenseRequired>
      /partners → <LicenseRequired>...</LicenseRequired>
      /settings → <SettingsPage />   ← always accessible
    </Routes>
  </BrowserRouter>
</AppProvider>
```

## Todo List
- [x] Remove LicenseGate from App.tsx
- [x] Add LicenseRequired to Encrypt/Decrypt/Partners routes
- [x] Verify Settings page loads without license
- [x] Verify Encrypt/Decrypt show license prompt without license
- [x] Compile check

## Success Criteria
- App starts, shows sidebar + header regardless of license state
- Settings fully functional without license
- Protected pages show LicenseNotFoundPage
