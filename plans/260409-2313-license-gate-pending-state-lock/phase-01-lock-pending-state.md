---
phase: 1
status: complete
priority: high
---

# Phase 1 — Lock Pending State in Gate + Conditional UX

## Overview
Remove "pending" from allowed states in `LicenseRequired`. Update `LicenseNotFoundPage` to show contextual message for "pending" (prompt token login) vs other states (prompt license export).

## Related Code Files
- Modify: `src/components/license-required.tsx`
- Modify: `src/components/license-not-found-page.tsx`

## Implementation Steps

### 1. `license-required.tsx` — Remove pending from allowed states

**Current (line 20):**
```tsx
if (license.licenseState !== "ok" && license.licenseState !== "pending") {
  return <LicenseNotFoundPage />;
}
```

**New:**
```tsx
if (license.licenseState !== "ok") {
  return <LicenseNotFoundPage reason={license.licenseState} />;
}
```

Pass `reason` prop so the page can show contextual messaging.

### 2. `license-not-found-page.tsx` — Conditional messaging for "pending"

Add `reason` prop (optional, backward compat). When `reason === "pending"`:
- Heading: "Token Login Required"
- Body: "Your communication key is set but token is not logged in. Please login your token to activate the license."
- Hide "Export Machine Credential" button (not relevant)
- Show hint: "Go to any encrypt/decrypt page after login to start working."

When `reason !== "pending"` (default): keep existing "License Not Found" + export credential UI unchanged.

**Pseudocode:**
```tsx
<!-- Updated: Validation Session 1 - Use LicenseState union type instead of string -->
interface Props {
  reason?: LicenseState;
}

export default function LicenseNotFoundPage({ reason }: Props) {
  const isPending = reason === "pending";
  // ...
  return (
    <div style={containerStyle}>
      <div style={cardStyle}>
        <div style={iconStyle} />
        <h2 style={headingStyle}>
          {isPending ? "Token Login Required" : "License Not Found"}
        </h2>
        <p style={bodyStyle}>
          {isPending
            ? "Your communication key is set but token is not logged in. Please login your token to activate the license."
            : "This application is not licensed for this machine. Use the button below to Export Machine Credential then contact your admin department."}
        </p>
        {!isPending && (
          <>
            <button ...>Export Machine Credential</button>
            {feedback && <p ...>{feedback}</p>}
          </>
        )}
        <p style={hintStyle}>
          {isPending
            ? "Use the Login Token button in the sidebar or Settings page."
            : "To import a license file, go to Settings."}
        </p>
      </div>
    </div>
  );
}
```

## Todo
- [x] Remove "pending" from allowed states in `LicenseRequired`
- [x] Pass `reason` prop to `LicenseNotFoundPage`
- [x] Add conditional rendering for "pending" vs default in `LicenseNotFoundPage`

## Success Criteria
- Navigate to encrypt/decrypt/partners with "pending" state → see "Token Login Required" page
- Navigate with "no_license"/"error" state → see existing "License Not Found" page
- Navigate with "ok" state → pages render normally
