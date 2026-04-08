# CAHTQT Client — Documentation

Welcome to the CAHTQT Client documentation. This directory contains comprehensive guides for understanding, developing, and deploying the application.

---

## Quick Navigation

### For New Developers
Start here to understand the project:
1. **[Project Overview & PDR](./project-overview-pdr.md)** — Requirements, design decisions, and acceptance criteria
2. **[System Architecture](./system-architecture.md)** — Components and license verification pipeline
3. **[Codebase Summary](./codebase-summary.md)** — Module organization and code structure
4. **[Code Standards](./code-standards.md)** — Naming conventions and implementation patterns

### For Feature Implementation
Reference these when building:
1. **[Code Standards](./code-standards.md)** — Rust/TypeScript conventions and common patterns
2. **[System Architecture](./system-architecture.md)** — Component responsibilities and data flow
3. **[License Verification Guide](./license-verification-guide.md)** — Deep dive into license system

### For Understanding License Verification
Complete reference for the 2F-HBLS system:
1. **[System Architecture](./system-architecture.md)** — High-level pipeline
2. **[License Verification Guide](./license-verification-guide.md)** — Step-by-step implementation details
3. **[Project Changelog](./project-changelog.md)** — Recent refactor and changes

### For Maintenance & Troubleshooting
Operational references:
1. **[Project Changelog](./project-changelog.md)** — Feature history and breaking changes
2. **[License Verification Guide](./license-verification-guide.md)** — Error scenarios and recovery
3. **[System Architecture](./system-architecture.md)** — Security considerations and future work

---

## Document Overview

| Document | Purpose | Audience |
|----------|---------|----------|
| **[project-overview-pdr.md](./project-overview-pdr.md)** | Functional/non-functional requirements, design decisions, acceptance criteria, implementation timeline | Architects, Product Managers, QA Engineers |
| **[system-architecture.md](./system-architecture.md)** | Component organization, license verification pipeline, data flow, security controls | Developers, Tech Leads |
| **[code-standards.md](./code-standards.md)** | Rust/TypeScript naming conventions, module structure, error handling, testing patterns | All Developers |
| **[codebase-summary.md](./codebase-summary.md)** | Project structure, module dependencies, file statistics, quick reference | New Developers, Code Reviewers |
| **[license-verification-guide.md](./license-verification-guide.md)** | Detailed walkthrough of license verification algorithm, integration points, testing checklist, debugging tips | Backend Developers, QA Engineers |
| **[project-changelog.md](./project-changelog.md)** | Feature history, breaking changes, migration guide, dependency updates | All Team Members |

---

## Key Concepts

### Two-Factor Hardware-Bound License System (2F-HBLS)

License verification happens in two phases at startup:

**Phase A: Token Verification**
- Initialize PKCS#11 library
- Get token serial and challenge-sign machine fingerprint
- Proves token with private key is present

**Phase B: License Binding**
- Read license.dat from disk (Base64-encoded payload + RSA signature)
- Extract public key from communication certificate (X.509)
- Verify RSA signature proves license issued by server
- Validate bindings: machine fingerprint, token serial, expiry

Result cached in AppState and served to frontend on demand.

### License Signature Verification Refactor (feature/license branch)

**Changed:** Public key source
- **Before:** Hardcoded `SERVER_PUBLIC_KEY_PEM` constant (guarded with `compile_error!`)
- **After:** Extracted at runtime from configurable communication certificate (stored in SQLite)

**Benefits:**
- Supports certificate rotation without recompilation
- No hardcoded secrets in source
- Flexible deployment (certificate imported in Settings)

**Impact:**
- Requires `communication_cert_path` in SQLite settings
- All builds enforce signature verification (no debug bypass)
- New `NoCommunicationCert` error status if path not configured

---

## File Paths Reference

### Source Code
```
src-tauri/src/
├── license/
│   ├── mod.rs                  # is_licensed(), extract_public_key_from_cert()
│   ├── error.rs                # LicenseStatus, LicenseError types
│   ├── payload.rs              # read_license_file(), verify_license_signature()
│   ├── machine.rs              # get_machine_fingerprint()
│   └── token.rs                # get_token_serial(), verify_token_challenge()
├── commands/
│   └── license.rs              # check_license(), export_machine_credential(), import_license_file()
└── lib.rs                       # Startup hook, AppState initialization
```

### Configuration
```
src-tauri/tauri.conf.json       # Tauri framework config
src-tauri/Cargo.toml            # Rust dependencies
```

### Documentation
```
docs/
├── README.md                   # This file
├── project-overview-pdr.md     # Requirements & design
├── system-architecture.md      # Components & pipeline
├── code-standards.md           # Coding conventions
├── codebase-summary.md         # Project structure
├── license-verification-guide.md  # Implementation details
├── project-changelog.md        # Feature history
└── journals/                   # Development notes
```

---

## Common Tasks

### I want to understand the license verification system
→ Start with [System Architecture](./system-architecture.md), then read [License Verification Guide](./license-verification-guide.md)

### I'm implementing a new Tauri command
→ Read [Code Standards](./code-standards.md) → Review similar command in `src-tauri/src/commands/license.rs` → Follow patterns

### I need to debug a license verification error
→ Check error scenario table in [License Verification Guide](./license-verification-guide.md) → Review error handling in code

### I'm onboarding a new developer
→ Send: [Project Overview](./project-overview-pdr.md) + [System Architecture](./system-architecture.md) + [Codebase Summary](./codebase-summary.md)

### I need to review a PR about license changes
→ Read [Project Changelog](./project-changelog.md) breaking changes section → Check acceptance criteria in [Project Overview](./project-overview-pdr.md)

### I'm preparing a deployment
→ Check [Project Overview](./project-overview-pdr.md) deployment checklist → Review [License Verification Guide](./license-verification-guide.md) error scenarios

---

## Documentation Standards

### Format
- Markdown (`.md` extension)
- Absolute file paths (for LLM tool compatibility)
- Code blocks with syntax highlighting
- Tables for structured data
- ASCII diagrams for complex flows

### Maintenance
- Update when code changes significantly
- Keep examples functional and tested
- No stale "TODO" markers (remove or fix)
- Cross-reference related docs

### File Size
- Individual docs typically 6–14KB
- Modular structure (split large docs into multiple files)
- Linked navigation for easy browsing

---

## Glossary

| Term | Definition |
|------|---|
| **2F-HBLS** | Two-Factor Hardware-Bound License System |
| **PKCS#11** | Cryptographic Token Interface standard |
| **X.509** | Digital certificate format standard |
| **SPKI** | Subject Public Key Info (certificate public key section) |
| **DER** | Distinguished Encoding Rules (binary certificate format) |
| **PEM** | Privacy-Enhanced Mail (Base64-encoded certificate format) |
| **RSA-PKCS1v15** | Asymmetric encryption and signature algorithm |
| **SHA256** | Cryptographic hash function (256-bit output) |
| **Machine Fingerprint** | SHA256(CPU ID + Board Serial + MAC addresses) |
| **AppState** | Tauri global application state (holds cached license info) |

---

## Related Resources

### In This Repository
- **README.md** (project root) — Project overview and setup instructions
- **CLAUDE.md** (project root) — Development rules and workflows
- **plans/** — Implementation plans with detailed phase-by-phase breakdowns

### External
- [Tauri Documentation](https://tauri.app/v1/guides/) — Desktop framework reference
- [PKCS#11 Specification](http://docs.oasis-open.org/pkcs11/) — Token interface standard
- [RFC 5280](https://tools.ietf.org/html/rfc5280) — X.509 certificate standard
- [PKCS#1 RFC 8017](https://tools.ietf.org/html/rfc8017) — RSA cryptography standard

---

## Document Versioning

| Document | Last Updated | Version |
|----------|---|---|
| project-overview-pdr.md | 2026-04-07 | 1.0 |
| system-architecture.md | 2026-04-07 | 1.0 |
| code-standards.md | 2026-04-07 | 1.0 |
| codebase-summary.md | 2026-04-07 | 1.0 |
| license-verification-guide.md | 2026-04-07 | 1.0 |
| project-changelog.md | 2026-04-07 | 1.0 |

---

## Questions?

1. **Technical Issue:** Check [License Verification Guide](./license-verification-guide.md) troubleshooting section
2. **Design Question:** Refer to [System Architecture](./system-architecture.md) design decisions
3. **Code Convention:** Check [Code Standards](./code-standards.md)
4. **Feature History:** See [Project Changelog](./project-changelog.md)

If not covered, create an issue or reach out to the development team.
