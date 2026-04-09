# License Gate Pending State Lock

**Date**: 2026-04-09 23:25
**Severity**: Medium
**Component**: License validation gate, token authentication
**Status**: Resolved

## What Happened

Implemented the "license gate lock for pending state" feature. Tightened access control by rejecting pending licenses at the gate layer instead of passing them through. When a user attempts to login with an expired/pending license, they now see differentiated UX prompting token re-authentication instead of a generic "not found" message.

## The Brutal Truth

Code review caught a state management mistake that could've shipped broken. The original plan said "keep modal open if revalidation fails," but the backend had already marked the token as valid. We'd trap users in a modal that can't close because the gate still shows pending. Frustrating oversight—we validated the design with three questions upfront but missed the interaction flow between token validation success and gate state refresh.

## Technical Details

**Changes**: 3 files, ~10 lines of logic
- Removed "pending" from `allowedStates` enum in license gate (only "ok" passes)
- `LicenseNotFoundPage`: Added conditional render — shows "Token Login Required" for pending, "License Not Found" otherwise
- `LoginTokenModal`: Wired `recheckLicense()` after token login completes
- TypeScript: 0 errors, clean compile

**State Fix**: Changed modal onSuccess to always call `onClose()` and let `recheckLicense()` run the gate revalidation. Gate then displays actual license state, not stale pending status.

## What We Tried

Initial implementation kept modal open on revalidation fail (per original spec). Code review flagged: "Token is valid on backend but gate still locked—user can't proceed." Solution: Trust the backend state, always proceed to gate refresh.

## Root Cause Analysis

Design didn't account for the backend-to-frontend state consistency gap. Token validation != license validation; we assumed they'd stay in sync but didn't wire the refresh. Should've traced token login → gate recheck → UI as a single flow during planning.

## Lessons Learned

- Always validate state transitions at review time, not commit time
- Backend-driven state changes need explicit frontend refresh triggers; don't assume async consistency
- The "keep modal open" pattern breaks when the condition it's waiting for is already satisfied upstream

## Next Steps

- Monitor production for pending state rejection behavior (ensure UX doesn't confuse users)
- Consider centralizing license gate refresh logic if more auth flows need it

**Commits**: dbc397f, 301c9b1
