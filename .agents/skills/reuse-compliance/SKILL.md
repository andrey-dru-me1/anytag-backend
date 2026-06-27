---
name: reuse-compliance
description: Add SPDX license/copyright headers to new files to maintain REUSE compliance
modeSlugs:
  - code
  - debug
  - architect
---
<!-- SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# REUSE Compliance Skill
<!-- REUSE-IgnoreStart -->

Add correct SPDX license and copyright headers to new or modified source files to maintain [REUSE Specification](https://reuse.software/) compliance.

## When to Use

This skill applies when creating a **new source file** or when an existing file is missing SPDX headers. The project CI enforces REUSE compliance via `.github/workflows/reuse.yml`, so every file must carry valid headers.

## Mandatory Convention (from AGENTS.md)

> EVERY source file MUST start with SPDX-FileCopyrightText and SPDX-License-Identifier comments. REUSE compliance is CI-checked.

This rule is **not enforced by `flutter analyze` or the Dart linter** — it's a project convention that must be followed manually.

## Header Format by File Type

| File type             | Comment style | Header template                                                                                                         |
| --------------------- | ------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Dart (`.dart`)        | `//`          | `// SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors` + `// SPDX-License-Identifier: AGPL-3.0-only`             |
| Markdown (`.md`)      | `<!-- -->`    | `<!-- SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors -->` + `<!-- SPDX-License-Identifier: AGPL-3.0-only -->` |
| Shell (`.sh`)         | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors` + `# SPDX-License-Identifier: AGPL-3.0-only`               |
| YAML (`.yml`/`.yaml`) | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors` + `# SPDX-License-Identifier: AGPL-3.0-only`               |
| TOML (`.toml`)        | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors` + `# SPDX-License-Identifier: AGPL-3.0-only`               |
| PowerShell (`.ps1`)   | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors` + `# SPDX-License-Identifier: AGPL-3.0-only`               |
| Kotlin (`.kt`)        | `//`          | `// SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors` + `// SPDX-License-Identifier: AGPL-3.0-only`             |
| Swift (`.swift`)      | `//`          | `// SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors` + `// SPDX-License-Identifier: AGPL-3.0-only`             |
| CMake (`.cmake`)      | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors` + `# SPDX-License-Identifier: AGPL-3.0-only`               |
| C++ (`.cpp`/`.h`)     | `//`          | `// SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors` + `// SPDX-License-Identifier: AGPL-3.0-only`             |
| C (`.c`)              | `//`          | `// SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors` + `// SPDX-License-Identifier: AGPL-3.0-only`             |
| Objective-C (`.m`)    | `//`          | `// SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors` + `// SPDX-License-Identifier: AGPL-3.0-only`             |
| XML / Plist           | `<!-- -->`    | `<!-- SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors -->` + `<!-- SPDX-License-Identifier: AGPL-3.0-only -->` |
## Using `reuse annotate` (Recommended)

The project uses `reuse` tool via `uv`:

```bash
# Single file
uv run reuse annotate --license AGPL-3.0-only --copyright "The Anytag Frontend Authors" lib/screens/new_screen.dart

# Multiple files with glob
uv run reuse annotate --license AGPL-3.0-only --copyright "The Anytag Frontend Authors" lib/screens/*.dart

# Dry-run to preview
uv run reuse annotate --license AGPL-3.0-only --copyright "The Anytag Frontend Authors" --dry-run lib/new_file.dart
```

## When NOT to Add Inline Headers

For **auto-generated files** (e.g., `pubspec.lock`, `.vscode/*.json`, `.metadata`), add the annotation to [`REUSE.toml`](../../../REUSE.toml) instead of inserting inline headers — the header would be overwritten on regeneration.

## Verification

Before committing, verify compliance:

```bash
uv run reuse lint
```

Expected output: **"All files are compliant!"**

## Step-by-Step

1. **Identify the file type** and determine the correct comment syntax.
2. **Add the SPDX header** at the very top of the file (first 2 lines):
   - Line 1: `SPDX-FileCopyrightText: 2026 The Anytag Frontend Authors`
   - Line 2: `SPDX-License-Identifier: AGPL-3.0-only`
3. **Use `reuse annotate`** for batch operations or if unsure about comment syntax.
4. **For auto-generated files**, add to [`REUSE.toml`](../../../REUSE.toml) instead.
5. **Run `uv run reuse lint`** to verify before pushing.
<!-- REUSE-IgnoreEnd -->

## See Also

- [`docs/REUSE.md`](../../../docs/REUSE.md) — Full REUSE compliance guide
- [`REUSE.toml`](../../../REUSE.toml) — Central REUSE configuration
- [`AGENTS.md`](../../../AGENTS.md) — Project conventions overview
