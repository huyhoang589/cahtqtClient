# License Validate at Startup

**Date**: 2026-04-08
**Severity**: Medium
**Component**: License Management / Routing
**Status**: Completed

## What Happened

Replaced full-app LicenseGate blocker with per-route LicenseRequired guard. Settings now always accessible, fixing the deadlock where unlicensed users couldn't configure certs or import licenses.

## Technical Details

- New hook: `useLicenseStatus` in AppContext for reactive license state
- New components: `license-required.tsx` (route guard), `license-not-found-page.tsx` (prompt page)
- `LicenseSection` now calls `recheckLicense()` after import — immediate unlock without polling
- Conditional comm cert banner (only shown when cert unconfigured)
- Deleted: `license-gate.tsx`, `license-screens.tsx`
- Code review fixes: un-awaited async call, unsafe type cast, missing `@keyframes pulse`

## Decisions Made

- Keep `/encrypt` as default redirect (not `/settings`) — users see license prompt immediately
- Manual recheck via context after import (avoids polling/backend changes)
- Text hint only on LicenseNotFoundPage (no Settings button — keeps workflow focused)
- Conditional banner reduces noise

## Open Question

`no_token` state collapsed into same UI as `no_license` — may need differentiation if still meaningful for future auth changes.

## Next Steps

Monitor if banner/route guard behavior matches user expectations. Consider differentiating token/license states if auth workflows change.
