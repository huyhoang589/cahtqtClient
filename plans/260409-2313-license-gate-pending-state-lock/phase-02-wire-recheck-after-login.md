---
phase: 2
status: complete
priority: high
---

# Phase 2 — Wire recheckLicense After Token Login

## Overview
After successful token login + revalidation, call `recheckLicense()` from app context to update React state. This triggers `LicenseRequired` re-render: "pending" → "ok" → gate opens.

## Related Code Files
- Modify: `src/components/login-token-modal.tsx`

## Implementation Steps

### 1. Import `useAppContext` and call `recheckLicense()` after revalidation

**Current (lines 36-38):**
```tsx
const result = await loginToken(pin);
// Revalidate license now that token session is available for .sf1 decrypt
revalidateLicense().catch(() => {});
```

**New:**
```tsx
const { license } = useAppContext();
// ... inside handleSubmit:
const result = await loginToken(pin);
// Revalidate license with token session, then refresh React state
await revalidateLicense();
await license.recheckLicense();
```

Key changes:
1. Add `import { useAppContext } from "../contexts/app-context";`
2. Call `useAppContext()` at component top level to get `license.recheckLicense`
3. Change `revalidateLicense().catch(() => {})` to `await revalidateLicense()` + `await license.recheckLicense()`
4. Both awaited so gate updates before modal closes

### 2. Error handling
<!-- Updated: Validation Session 1 - Show error in modal on revalidation failure instead of closing -->

If `revalidateLicense()` fails, do NOT close the modal. Show an error message so the user can retry. Only proceed to `recheckLicense()` + close on success:

```tsx
try {
  await revalidateLicense();
  await license.recheckLicense();
  // success → close modal
} catch {
  setError("Token logged in but license validation failed. Please try again.");
  // modal stays open, user can retry
}
```

## Todo
- [x] Import `useAppContext` in `login-token-modal.tsx`
- [x] Get `license.recheckLicense` from context
- [x] Await `revalidateLicense()` then `recheckLicense()` in handleSubmit
- [x] Verify modal closes after state refresh

## Success Criteria
- Login token with valid PIN → license state transitions from "pending" to "ok"
- `LicenseRequired` gate re-renders → encrypt/decrypt pages become accessible
- Login with invalid PIN → error shown, license state unchanged
- If revalidation fails → recheckLicense still runs, gate shows current state
