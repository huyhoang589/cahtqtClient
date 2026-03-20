# CAHTQT Code Standards & Guidelines

**Version:** 1.0.0
**Last Updated:** 2026-02-20

## Overview

This document defines coding standards for CAHTQT across Rust (backend) and TypeScript/React (frontend) layers. All contributors must follow these standards to maintain consistency and quality.

## General Principles

1. **YAGNI (You Aren't Gonna Need It)** - Don't implement speculative features
2. **KISS (Keep It Simple, Stupid)** - Prefer simple, readable code over clever
3. **DRY (Don't Repeat Yourself)** - Extract common logic into reusable modules
4. **Readability > Performance** - Optimize only when profiling shows bottleneck
5. **Tests First** - Write tests alongside implementation

## Rust Backend Standards

### File Naming & Organization
- **Case:** snake_case for files and module names
- **Max Size:** 200 lines per file (split into modules if larger)
- **Structure:**
  ```
  src/
  ├── lib.rs              (main library entry, state mgmt)
  ├── main.rs             (binary entry, delegates to lib.rs)
  ├── models.rs           (data structures)
  ├── commands/           (Tauri command handlers)
  │   ├── mod.rs
  │   ├── settings.rs
  │   ├── groups.rs
  │   ├── encrypt.rs
  │   ├── decrypt.rs
  │   └── logs.rs
  ├── db/                 (database repositories)
  │   ├── mod.rs
  │   ├── settings_repo.rs
  │   ├── groups_repo.rs
  │   ├── recipients_repo.rs
  │   └── logs_repo.rs
  └── [service modules]
      ├── dll_wrapper.rs
      ├── cert_parser.rs
      ├── pkcs11_service.rs
      └── dll_error.rs
  ```

### Code Style

#### Formatting
- **Editor Config:** VS Code defaults (2 spaces for indentation is fine)
- **Line Length:** 100 characters (soft limit, hard at 120)
- **Imports:** Group by: std, external crates, local modules
- **Blank Lines:** 2 between top-level items, 1 within modules

#### Example Structure
```rust
// Standard file header (file-specific doc comment if needed)

// Imports: std, then external, then local
use std::path::PathBuf;

use sqlx::SqlitePool;
use tauri::Manager;

use crate::models::{Group, Recipient};
use crate::db;

/// Public function documentation (rustdoc)
pub async fn create_group(db: &SqlitePool, name: String) -> Result<String, AppError> {
    // Implementation
}

/// Private helper — brief comment only
fn validate_group_name(name: &str) -> bool {
    // Implementation
}
```

### Error Handling

#### Custom Error Types
- Define per module (e.g., `dll_error.rs` for FFI errors)
- Use `thiserror` crate for error definitions
- Always include context (file path, SQL query, etc.)

#### Example
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DllError {
    #[error("DLL not found at {path}")]
    NotFound { path: String },

    #[error("FFI call failed: {reason}")]
    CallFailed { reason: String },

    #[error("Invalid certificate: {0}")]
    InvalidCert(String),
}
```

#### Usage Pattern
```rust
pub fn load_dll(path: &str) -> Result<CryptoDll, DllError> {
    let lib = unsafe { libloading::Library::new(path) }
        .map_err(|e| DllError::NotFound {
            path: path.to_string(),
        })?;
    // ...
}
```

### Async/Await

#### Rules
- Use `tokio::spawn` for background tasks only
- Use `block_on` in setup (main.rs), nowhere else
- All DB operations are async (await required)
- Tauri commands are async by default

#### Example
```rust
#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> Result<SettingsMap, String> {
    db::settings_repo::get_all(&state.db)
        .await
        .map_err(|e| e.to_string())
}
```

### Security Standards

#### PIN Handling
- Accept `Vec<u8>` or `&[u8]`, never `String`
- Zero immediately after use with `zeroize`
- Never log or print PIN values

```rust
pub async fn decrypt_batch(
    pin: Vec<u8>,
    // ...
) -> Result<Vec<PathBuf>, String> {
    // Use pin
    let result = dll.decrypt_files(&pin)?;

    // Zero before return
    zeroize::Zeroize::zeroize(&mut pin);
    Ok(result)
}
```

#### Certificate Validation
- Parse X.509 via x509-parser (no unsafe parsing)
- Check cert validity dates
- Log cert subject on import (for audit)

#### Database Security
- Use SQLx parameterized queries (never string concatenation)
- No SQL injection possible (compile-time checked)

### Testing

#### Test Organization
- Tests in `#[cfg(test)]` modules at bottom of each file
- File name: `{module}_tests` if separate file
- Use `#[tokio::test]` for async tests

#### Example
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_group() {
        // Setup (fixture or builder)
        let db = setup_test_db().await;

        // Execute
        let group_id = create_group(&db, "Test".to_string()).await.unwrap();

        // Assert
        assert!(!group_id.is_empty());
    }
}
```

### Documentation

#### Rustdoc Standards
- All public functions must have `/// Doc comment`
- Include example if behavior isn't obvious
- Document errors in `# Errors` section
- Document panics only if possible

#### Example
```rust
/// Encrypts files for recipients in a group.
///
/// # Arguments
/// * `db` - SQLite connection pool
/// * `group_id` - ID of recipient group
/// * `file_paths` - Paths to files to encrypt
///
/// # Returns
/// Vec of output file paths on success.
///
/// # Errors
/// Returns `DllError` if DLL is not loaded or encrypt fails.
pub async fn encrypt_batch(
    db: &SqlitePool,
    group_id: &str,
    file_paths: Vec<PathBuf>,
) -> Result<Vec<PathBuf>, DllError> {
    // ...
}
```

#### Inline Comments
- Explain "why", not "what"
- One comment per complex logic block
- Keep comments updated with code

```rust
// GOOD: explains intent
// Group recipients by token to avoid multiple PIN prompts
let grouped = group_recipients_by_token(recipients);

// BAD: restates code
// Create iterator and map
let iter = recipients.iter().map(...);
```

### Dependencies

#### Approved Libraries (Cargo.toml)
- **Core:** tauri 2, sqlx 0.8, tokio 1
- **FFI:** libloading 0.8
- **Crypto:** x509-parser 0.16, cryptoki 0.6, zeroize 1.8
- **Serialization:** serde 1.0, serde_json 1.0
- **Error Handling:** thiserror 1
- **Utilities:** uuid 1, dirs 5

Do NOT add dependencies without discussion:
- Evaluate size impact
- Check maintenance status
- Prefer std library if available

### Build Configuration

#### Release Profile
```toml
[profile.release]
panic = "abort"        # Faster, no unwinding
codegen-units = 1      # Better optimization
lto = true             # Link-time optimization
opt-level = "s"        # Optimize for size
strip = true           # Remove debug symbols
```

#### Debug Profile
```toml
[profile.dev]
incremental = true     # Faster rebuilds
```

## TypeScript / React Frontend Standards

### File Naming & Organization
- **Case:**
  - Files: kebab-case
  - Components: PascalCase (e.g., `app-sidebar.tsx` exports `AppSidebar`)
  - Utilities: camelCase (e.g., `formatDate.ts`)
- **Max Size:** 200 lines per component (split if larger)

#### Directory Structure
```
src/
├── main.tsx            (Vite entry point)
├── App.tsx             (Router, layout)
├── pages/              (Page components)
│   ├── EncryptPage.tsx
│   ├── DecryptPage.tsx
│   ├── GroupsPage.tsx
│   └── SettingsPage.tsx
├── components/         (Reusable UI components)
│   ├── app-sidebar.tsx
│   ├── status-bar.tsx
│   ├── file-list-panel.tsx
│   └── [other components]
├── utils/              (Helpers, not yet created)
│   ├── format-date.ts
│   └── [other utilities]
└── types/              (Shared TypeScript types, if needed)
    └── index.ts
```

### Code Style

#### Formatting
- **Indentation:** 2 spaces
- **Line Length:** 100 characters (soft), 120 (hard)
- **Quotes:** Double quotes for strings
- **Semicolons:** Required (enforced by TypeScript)
- **Trailing Commas:** Yes (multiline objects/arrays)

#### Example Component
```typescript
import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Props {
  groupId: string;
  onSuccess?: () => void;
}

export const AddRecipientDialog: React.FC<Props> = ({ groupId, onSuccess }) => {
  const [alias, setAlias] = useState("");
  const [loading, setLoading] = useState(false);

  const handleAdd = async () => {
    setLoading(true);
    try {
      await invoke("add_recipient", { groupId, alias });
      onSuccess?.();
    } catch (error) {
      console.error("Add recipient failed:", error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      {/* Component JSX */}
    </div>
  );
};
```

### Component Patterns

#### Functional Components Only
- Use React hooks (useState, useEffect, useContext)
- No class components
- Extract custom hooks for reusable logic

#### Props Definition
```typescript
interface PageProps {
  // Required props
  title: string;
  items: Item[];

  // Optional props
  onSelect?: (id: string) => void;
  maxItems?: number;
}

export const MyPage: React.FC<PageProps> = ({
  title,
  items,
  onSelect,
  maxItems = 10,
}) => {
  // ...
};
```

#### Tauri Command Invocation
```typescript
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Command with result
const handleEncrypt = async () => {
  try {
    const result = await invoke<EncryptResult>("encrypt_batch", {
      groupId,
      filePaths,
    });
    console.log("Encrypted:", result);
  } catch (error) {
    console.error("Encryption failed:", error);
  }
};

// Event listener
useEffect(() => {
  const unlisten = listen<ProgressPayload>("encrypt_progress", (event) => {
    setProgress(event.payload);
  });

  return () => {
    unlisten.then((fn) => fn());
  };
}, []);
```

### Error Handling

#### User-Facing Errors
- Catch errors, show user-friendly toast/dialog
- Never expose raw error messages to users
- Log full error to console for debugging

```typescript
const handleSave = async () => {
  try {
    await invoke("set_setting", { key, value });
    showToast("Setting saved");
  } catch (error: any) {
    console.error("Save failed:", error);
    showError("Failed to save setting. Please try again.");
  }
};
```

#### Type-Safe Error Handling
```typescript
interface ApiError {
  message: string;
  code?: string;
}

const isApiError = (error: unknown): error is ApiError => {
  return (
    typeof error === "object" &&
    error !== null &&
    "message" in error
  );
};

try {
  // ...
} catch (error) {
  if (isApiError(error)) {
    console.error(`Error ${error.code}:`, error.message);
  } else {
    console.error("Unknown error:", error);
  }
}
```

### TypeScript Best Practices

#### Type Definitions
- Always define types for complex objects
- Use interfaces for object shapes, types for primitives
- Avoid `any` (use `unknown` with type guard if needed)

```typescript
// Good
interface Recipient {
  id: string;
  alias: string;
  certExpiry: Date;
}

// Bad
const recipients: any[] = [];
```

#### Type Imports
```typescript
import type { AppState } from "./types";
import { AppState } from "./state";  // Runtime import

// Or use `import type` for types only
import type { ProgressPayload } from "./events";
```

#### Nullable/Optional Values
```typescript
// Use union with null
const maybeId: string | null = null;

// Use optional fields
interface User {
  name: string;
  email?: string;
}

// Handle nullish with nullish coalescing or optional chaining
const displayName = user?.name ?? "Unknown";
```

### Testing

#### Jest Configuration (if added)
- Test file: `{component}.test.tsx`
- Test structure:
  ```typescript
  import { render, screen } from "@testing-library/react";
  import { MyComponent } from "./my-component";

  describe("MyComponent", () => {
    it("renders heading", () => {
      render(<MyComponent />);
      expect(screen.getByRole("heading")).toBeInTheDocument();
    });
  });
  ```

### State Management

#### Current Approach
- No global state manager (keep it simple)
- Component-level state via useState
- Props drilling acceptable for CAHTQT's size
- Consider context API if 3+ levels of drilling

#### Example Context (if needed)
```typescript
interface AppContextValue {
  dllLoaded: boolean;
  pkcs11Available: boolean;
}

export const AppContext = React.createContext<AppContextValue | null>(null);

export const useAppStatus = () => {
  const ctx = React.useContext(AppContext);
  if (!ctx) throw new Error("useAppStatus must be used inside AppProvider");
  return ctx;
};
```

### Performance

#### Optimization Checklist
- Use `React.memo` for expensive components only
- Use `useMemo` only when profiling shows benefit
- `useCallback` for event handlers passed to children
- Avoid inline function definitions in render

```typescript
// Good: memoized if expensive
const ProgressBar = React.memo(({ value }: Props) => {
  return <div>{value}%</div>;
});

// Good: stable callback
const handleClick = useCallback(() => {
  onSelect(id);
}, [id, onSelect]);

// Bad: re-created on every render
<button onClick={() => onSelect(id)}>Select</button>
```

### Styling

#### Current Approach
- Inline styles or Tailwind (if configured)
- CSS modules if global stylesheet needed
- Avoid CSS-in-JS runtime libraries (use Tailwind or plain CSS)

### Documentation

#### Component Comments
```typescript
/**
 * Displays a list of recipients for a group.
 * Shows certificate details in a popover on hover.
 *
 * @param groupId - ID of group to display recipients for
 * @param onRemove - Callback when user deletes a recipient
 */
export const RecipientTable: React.FC<RecipientTableProps> = ({
  groupId,
  onRemove,
}) => {
  // ...
};
```

#### Inline Comments
- Explain non-obvious logic only
- Keep updated with code

## API Contract Standards

### Command Naming
- **Verb-based:** `get_settings`, `create_group`, `encrypt_batch`
- **Snake_case:** Consistent with Rust convention

### Request/Response Types

#### Example: Settings Command
**Request (TypeScript):**
```typescript
interface SetSettingRequest {
  key: string;     // "dll_path" | "sender_name" | ...
  value: string;   // Setting value
}
```

**Response:**
```typescript
interface SetSettingResponse {
  success: boolean;
  error?: string;
}
```

#### Example: Encrypt Batch Command
**Request:**
```typescript
interface EncryptBatchRequest {
  groupId: string;
  filePaths: string[];
  format?: "V1" | "V2";  // Default: "V1"
}
```

**Response (event-based):**
```typescript
interface EncryptProgressPayload {
  currentFile: number;
  totalFiles: number;
  currentRecipient: number;
  totalRecipients: number;
}

// Final response via resolve
interface EncryptBatchResponse {
  outputPaths: string[];
  totalOperations: number;
}
```

### Event Naming
- **PascalCase suffix:** `EncryptProgress`, `DecryptProgress`
- Emitted before, during, and after operations

## Version Control & Commits

### Commit Message Format (Conventional Commits)
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:** `feat`, `fix`, `refactor`, `docs`, `chore`, `test`
**Scope:** Component/module name
**Subject:** Imperative mood, lowercase, no period

#### Examples
```
feat(encrypt): add M×N batch progress tracking

fix(settings): validate DLL path before saving

refactor(db): split recipients_repo into smaller functions

docs(architecture): update system overview diagram
```

### Pre-Commit Checklist
- [ ] Code compiles (Rust: `cargo check`, TS: `tsc`)
- [ ] No console.error/log left (dev only)
- [ ] No uncommitted sensitive files (.env, credentials)
- [ ] Tests pass (if applicable)

## Review Checklist

Code reviewers should verify:
- [ ] Follows naming conventions (snake_case Rust, PascalCase components)
- [ ] Error handling present (all async ops try/catch)
- [ ] No hardcoded paths or magic numbers
- [ ] Documentation updated (rustdoc, JSDoc, architecture docs)
- [ ] No security regressions (PIN handling, SQL injection, etc.)
- [ ] Type safety (no `any` in TS, proper error types in Rust)
- [ ] Tests added (if applicable)

## Tools & Commands

### Rust
```bash
# Check for errors (no build)
cargo check

# Build debug
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Format (if rustfmt installed)
cargo fmt

# Lint
cargo clippy
```

### TypeScript/React
```bash
# Type check
tsc

# Build
npm run build

# Dev server
npm run dev

# Format (if prettier installed)
npx prettier --write src/

# Lint (if ESLint configured)
npx eslint src/
```

## Continuous Improvement

- Review code standards quarterly
- Update based on Rust/TypeScript language changes
- Maintain consistency across the codebase
- Document exceptions (e.g., why `unsafe` used in dll_wrapper.rs)
