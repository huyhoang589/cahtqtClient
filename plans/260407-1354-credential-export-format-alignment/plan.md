---
title: "Credential Export Format Alignment"
description: "Align export_machine_credential output to server-expected spec format"
status: done
priority: P1
effort: 1h
branch: feature/license
tags: [license, credential, export, spec-compliance]
created: 2026-04-07
blockedBy: []
blocks: []
---

# Credential Export Format Alignment

## Overview

Update `export_machine_credential` command to output the exact JSON format expected by the CAHTQT PKI Server. Current impl exports hashed fingerprint + extra fields; server expects raw hardware IDs only.

## Current vs Target

| Field | Current | Target |
|-------|---------|--------|
| `token_serial` | Yes | Yes |
| `cpu_id` | No (hashed) | Yes (raw) |
| `board_serial` | No (hashed) | Yes (raw) |
| `registered_at` | No (`exported_at`) | Yes (ISO 8601) |
| `machine_fingerprint` | Yes | Remove |
| `user_name` | Yes | Remove |
| `app_version` | Yes | Remove |

## Phases

| # | Phase | Status | Effort | File |
|---|-------|--------|--------|------|
| 1 | Update Rust command + types | done | 40min | [phase-01](phase-01-update-rust-command.md) |
| 2 | Update TypeScript types | done | 10min | [phase-02](phase-02-update-typescript-types.md) |

## Key Decisions

1. `machine.rs` already exposes `get_cpu_id()` and `get_board_serial()` as pub — no changes needed there
2. `get_machine_fingerprint()` stays unchanged — still used by license verification pipeline
3. `read_first_cert_cn` helper removed (only used for `user_name` field being dropped)
4. Frontend only uses `saved_path` from result — minimal UI impact
5. `MachineCredentialResult` simplified: only `saved_path` needed

## Dependencies

- Existing plan `260407-1013-license-module-settings-page` (status: done) — no conflict

## Validation Log

### Session 1 — 2026-04-07
**Trigger:** Initial plan validation before implementation
**Questions asked:** 3

#### Questions & Answers

1. **[Edge case]** When cpu_id or board_serial cannot be read (returns None), what should the export do?
   - Options: Export empty string (Recommended) | Fail the export with error | Export null value
   - **Answer:** Export empty string
   - **Rationale:** Server-side handles validation; keeps export non-blocking. `unwrap_or_default()` is appropriate.

2. **[Timestamp]** The spec shows timestamp as '2025-01-01T00:00:00Z' but chrono's to_rfc3339() produces '2026-04-07T14:00:00.123456+00:00' (with fractional seconds and +00:00 offset). Should we match the spec's exact format?
   - Options: Match spec exactly (Recommended) | Use to_rfc3339() as-is
   - **Answer:** Match spec exactly
   - **Rationale:** Avoid server parsing issues. Use explicit `format!` with `%Y-%m-%dT%H:%M:%SZ` — no fractional seconds, trailing Z.

3. **[Scope]** The brainstorm listed changes to license-screens.tsx and LicenseSection.tsx, but the plan says no changes needed (frontend only uses saved_path). Have you confirmed these files don't reference token_serial or user_name from the credential result?
   - Options: Yes, plan is correct | Not sure, verify first
   - **Answer:** Yes, plan is correct
   - **Rationale:** Frontend only consumes `saved_path` — scope confirmed minimal.

#### Confirmed Decisions
- **Empty string for missing hardware IDs**: `unwrap_or_default()` — server validates
- **Exact spec timestamp format**: Use `format!("{}", utc.format("%Y-%m-%dT%H:%M:%SZ"))` instead of `to_rfc3339()`
- **No frontend file changes**: Only `types/index.ts` needs updating

#### Action Items
- [ ] Update phase-01 timestamp implementation to use explicit format instead of `to_rfc3339()`

#### Impact on Phases
- Phase 1: Update timestamp code from `chrono::Utc::now().to_rfc3339()` to explicit `Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()`
