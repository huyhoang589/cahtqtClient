---
title: "License Validate at Startup — Per-Page Guard"
description: "Replace LicenseGate full-app blocker with per-route LicenseRequired guard so Settings remains accessible when unlicensed"
status: complete
priority: P1
effort: 3h
branch: feature/licenseValidateStartup
tags: [license, startup, routing, ux]
created: 2026-04-08
---

# License Validate at Startup v1

## Background
Current `LicenseGate` wraps entire app — blocks Settings when license invalid. Users can't configure comm cert or import license. Deadlock.

## Solution
Per-page route guard. Settings always accessible. Encrypt/Decrypt/Partners show license prompt.

## Phases

| # | Phase | Status | File |
|---|-------|--------|------|
| 1 | Add license state to AppContext | Complete | [phase-01](phase-01-license-context.md) |
| 2 | Create LicenseRequired route guard + LicenseNotFoundPage | Complete | [phase-02](phase-02-license-required-guard.md) |
| 3 | Refactor App.tsx routing | Complete | [phase-03](phase-03-refactor-app-routing.md) |
| 4 | Add cert info banner to LicenseSection | Complete | [phase-04](phase-04-license-section-banner.md) |
| 5 | Cleanup old LicenseGate files | Complete | [phase-05](phase-05-cleanup.md) |

## Key Dependencies
- Rust backend license commands unchanged (check_license, get_license_info, export_machine_credential, import_license_file)
- Existing AppContext hooks (useTokenStatus, useSettingsStore) remain intact

## Success Criteria
- Settings accessible regardless of license state
- Encrypt/Decrypt/Partners show license prompt when unlicensed
- Export Machine Credential works from prompt page
- After importing license in Settings, protected pages become accessible
- Info banner in LicenseSection about comm cert requirement

## Validation Log

### Session 1 — 2026-04-08
**Trigger:** Initial plan validation before implementation
**Questions asked:** 4

#### Questions & Answers

1. **[Architecture]** When unlicensed, the default route `/` redirects to `/encrypt` which will show the license prompt. Should it redirect to `/settings` instead so users land somewhere useful?
   - Options: Keep /encrypt redirect (Recommended) | Redirect to /settings | Conditional redirect
   - **Answer:** Keep /encrypt redirect
   - **Rationale:** User sees license prompt immediately, understands they need a license. Consistent with current behavior.

2. **[Architecture]** After importing a license in Settings, how should protected pages (Encrypt/Decrypt/Partners) detect the change?
   - Options: Manual recheck via context (Recommended) | Auto-poll on interval | Tauri event listener
   - **Answer:** Manual recheck via context
   - **Rationale:** LicenseSection calls recheckLicense() after successful import. Protected pages re-render via AppContext. No backend changes needed.

3. **[UX]** The LicenseNotFoundPage only has 'Export Machine Credential' button and a text hint to go to Settings. Should it include a direct link/button to navigate to Settings?
   - Options: Text hint only (Recommended) | Add Settings link | Add Settings button
   - **Answer:** Text hint only
   - **Rationale:** Keeps the page focused on export workflow per spec. Hint text sufficient.

4. **[Scope]** Phase 4 adds a static info banner about comm cert requirement. Should this banner be conditional (only show when cert is not configured)?
   - Options: Always show (Recommended) | Show only when no cert | Skip banner entirely
   - **Answer:** Show only when no cert
   - **Rationale:** More polished UX — banner disappears after cert configured, reducing noise. Requires checking comm cert status.

#### Confirmed Decisions
- Default route: Keep `/encrypt` redirect — consistent UX
- License sync: Manual recheck via AppContext after import — no polling/backend changes
- Navigation UX: Text hint only on LicenseNotFoundPage — focused workflow
- Banner logic: Conditional — show only when comm cert not configured

#### Action Items
- [x] Phase 4: Change banner from static to conditional (check comm cert status)
- [x] Phase 1 or 2: Ensure recheckLicense() is accessible for LicenseSection to call after import

#### Impact on Phases
- Phase 4: Banner must check comm cert existence and only render when cert is missing. Needs access to cert status (likely from AppContext or a Tauri command).
