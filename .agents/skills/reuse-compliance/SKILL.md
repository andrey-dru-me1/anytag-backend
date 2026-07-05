---
name: reuse-compliance
description: Add SPDX license/copyright headers to new files to maintain REUSE compliance
modeSlugs:
  - code
  - debug
  - architect
---
<!-- SPDX-FileCopyrightText: 2026 The Anytag Backend Authors -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# REUSE Compliance Skill
<!-- REUSE-IgnoreStart -->

Add correct SPDX license and copyright headers to new or modified source files to maintain [REUSE Specification](https://reuse.software/) compliance.

## When to Use

This skill applies when creating a **new source file** or when an existing file is missing SPDX headers. The project CI enforces REUSE compliance via `.github/workflows/reuse.yml`, so every file must carry valid headers.

## Mandatory Convention (from AGENTS.md)

> EVERY source file MUST start with SPDX-FileCopyrightText and SPDX-License-Identifier comments. REUSE compliance is CI-checked.

This rule is **not enforced by `cargo check` or the Rust compiler** — it's a project convention that must be followed manually.

## Header Format by File Type

| File type             | Comment style | Header template                                                                                                         |
| --------------------- | ------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Rust (`.rs`)          | `//`          | `// SPDX-FileCopyrightText: 2026 The Anytag Backend Authors` + `// SPDX-License-Identifier: AGPL-3.0-only`             |
| Markdown (`.md`)      | `<!-- -->`    | `<!-- SPDX-FileCopyrightText: 2026 The Anytag Backend Authors -->` + `<!-- SPDX-License-Identifier: AGPL-3.0-only -->` |
| Shell (`.sh`)         | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Backend Authors` + `# SPDX-License-Identifier: AGPL-3.0-only`               |
| YAML (`.yml`/`.yaml`) | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Backend Authors` + `# SPDX-License-Identifier: AGPL-3.0-only`               |
| TOML (`.toml`)        | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Backend Authors` + `# SPDX-License-Identifier: AGPL-3.0-only`               |
| Dockerfile            | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Backend Authors` + `# SPDX-License-Identifier: AGPL-3.0-only`               |
| C++ (`.cpp`/`.h`)     | `//`          | `// SPDX-FileCopyrightText: 2026 The Anytag Backend Authors` + `// SPDX-License-Identifier: AGPL-3.0-only`             |
| C (`.c`)              | `//`          | `// SPDX-FileCopyrightText: 2026 The Anytag Backend Authors` + `// SPDX-License-Identifier: AGPL-3.0-only`             |
| XML / Plist           | `<!-- -->`    | `<!-- SPDX-FileCopyrightText: 2026 The Anytag Backend Authors -->` + `<!-- SPDX-License-Identifier: AGPL-3.0-only -->` |
## Using `reuse annotate` (Recommended) — Examples


```bash
# Single file
reuse annotate --license AGPL-3.0-only --copyright "The Anytag Backend Authors" src/handlers/new_handler.rs

# Multiple files with glob
reuse annotate --license AGPL-3.0-only --copyright "The Anytag Backend Authors" src/handlers/*.rs

# Dry-run to preview
reuse annotate --license AGPL-3.0-only --copyright "The Anytag Backend Authors" --dry-run src/new_file.rs
```

## When NOT to Add Inline Headers

For **auto-generated files** (e.g., `Cargo.lock`, `.vscode/*.json`), add the annotation to [`REUSE.toml`](../../../REUSE.toml) instead of inserting inline headers — the header would be overwritten on regeneration.

## Verification

Before committing, verify compliance:

```bash
reuse lint
```

Expected output: **"All files are compliant!"**

## Step-by-Step

1. **Identify the file type** and determine the correct comment syntax.
2. **Add the SPDX header** at the very top of the file (first 2 lines):
   - Line 1: `SPDX-FileCopyrightText: 2026 The Anytag Backend Authors`
   - Line 2: `SPDX-License-Identifier: AGPL-3.0-only`
3. **Use `reuse annotate`** for batch operations or if unsure about comment syntax.
4. **For auto-generated files**, add to [`REUSE.toml`](../../../REUSE.toml) instead.
5. **Run `reuse lint`** to verify before pushing.
<!-- REUSE-IgnoreEnd -->

## See Also

- [`docs/REUSE.md`](../../../docs/REUSE.md) — Full REUSE compliance guide
- [`REUSE.toml`](../../../REUSE.toml) — Central REUSE configuration
- [`AGENTS.md`](../../../AGENTS.md) — Project conventions overview
