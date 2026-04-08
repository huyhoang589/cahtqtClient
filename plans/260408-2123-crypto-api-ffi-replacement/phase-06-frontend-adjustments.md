## Phase 6: Frontend Adjustments

### Overview
- **Priority**: Medium
- **Status**: complete
- **Description**: Minimal frontend changes to align with backend API changes.

### Related Code Files
- **Modify**: `src/lib/tauri-api.ts`, `src/hooks/use-encrypt.ts`, `src/hooks/use-decrypt.ts`
- **Depends on**: Phases 4-5

### Implementation Steps

1. **`tauri-api.ts`** (~lines 141-156):
   - `encryptBatch()`: No signature change needed (backend still takes same Tauri command args)
   - `decryptBatch()`: **Remove `partnerName` param** — confirmed not needed (DLL handles output path via out_path_buf)
   <!-- Updated: Validation Session 1 - Remove partnerName from decryptBatch() -->

2. **`use-encrypt.ts`** (~lines 6-52):
   - Progress total may change: was M×N pairs, now just M files for final results
   - During encryption, progress callback still fires per (file, recipient) pair
   - Verify progress tracking logic handles the difference
   - No structural changes expected

3. **`use-decrypt.ts`** (~lines 6-50):
   - Minimal changes expected
   - Verify progress events still match expected shape

4. **Type definitions** (if any in tauri-api.ts):
   - Update `EncryptResult` if total/success_count semantics changed
   - `DecryptResult` likely unchanged

### Todo
- [x] Review decryptBatch() partnerName usage
- [x] Verify progress tracking in use-encrypt.ts
- [x] Verify progress tracking in use-decrypt.ts
- [x] Update type definitions if needed

### Success Criteria
- Frontend compiles without errors
- Progress tracking displays correctly
- Encrypt/decrypt flows work end-to-end
