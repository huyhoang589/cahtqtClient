---
phase: 2
title: "Update TypeScript Types"
status: done
priority: P1
effort: 10min
---

# Phase 2: Update TypeScript Types

## Related Code Files

### Modify
- `src/types/index.ts` — Update `MachineCredentialResult` interface

### No Changes Needed
- `src/components/license-screens.tsx` — Only uses `result.saved_path` (line 53)
- `src/pages/Settings/LicenseSection.tsx` — Only uses `result.saved_path` (line 42)
- `src/lib/tauri-api.ts` — Type import remains valid

## Implementation Steps

### 1. Update `MachineCredentialResult` interface (line 240-244)

```typescript
// BEFORE
export interface MachineCredentialResult {
  saved_path: string;
  token_serial: string;
  user_name: string;
}

// AFTER
export interface MachineCredentialResult {
  saved_path: string;
}
```

## Todo List

- [ ] Update `MachineCredentialResult` in `types/index.ts`
- [ ] Verify no other references to `token_serial` or `user_name` from credential result
- [ ] Run TypeScript compilation check

## Success Criteria

- TypeScript compiles without errors
- Frontend renders correctly (no runtime errors)
