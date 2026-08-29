<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# REUSE Compliance Guide

This project follows the [REUSE Specification](https://reuse.software/) to standardize license and copyright information across all files.

## License

The project is licensed under **AGPL-3.0-only**. See [`LICENSE`](LICENSE) (symlinked to [`LICENSES/AGPL-3.0-only.txt`](LICENSES/AGPL-3.0-only.txt)) for the full license text.

## REUSE Configuration

- [`REUSE.toml`](REUSE.toml) — central configuration that bulk-annotates files unable to carry inline SPDX headers
- [`LICENSES/`](LICENSES/) — directory containing the full text of every license used in the project
- Individual source files carry inline SPDX headers at the top

### When to use `REUSE.toml` vs. inline headers

| Approach                    | When to use                                                                          | Examples                                                      |
| --------------------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------- |
| **Inline SPDX header**      | Source files you create and edit                                                     | `.rs`, `.md`, `.sh`, `.sql`, `.toml`, `.yml`                  |
| **`REUSE.toml` annotation** | Auto-generated files, lock files, or files where inline comments would break tooling | `Cargo.lock`, `flake.lock`, `src/schema.rs`, `.vscode/*.json` |

If a file is auto-generated (e.g., by Diesel or `cargo`), add it to [`REUSE.toml`](REUSE.toml) instead of inserting an inline header — the header would be overwritten on regeneration.

## Adding a New File

Every new file must include an SPDX header identifying its copyright and license. Use [`reuse annotate`](https://reuse.readthedocs.io/en/latest/manpage.html#annotate) to add headers. `reuse` is provided by mise (`pipx:reuse`); run it through `just` (safe without a mise-activated shell) or directly inside one:

```bash
# Single file
just reuse annotate --license AGPL-3.0-only --copyright "The Anytag Backend Authors" src/handlers/new_module.rs

# Multiple files with a glob pattern
just reuse annotate --license AGPL-3.0-only --copyright "The Anytag Backend Authors" src/handlers/*.rs

# Dry-run to preview changes first
just reuse annotate --license AGPL-3.0-only --copyright "The Anytag Backend Authors" --dry-run src/new_file.rs
```

> Bare `reuse` works only in a mise-activated shell (VS Code terminal with the mise
> extension, or `eval "$(mise activate)"` in your shell config). Otherwise use the
> `just reuse ...` wrapper above.

This automatically inserts the correct SPDX header using the appropriate comment syntax for each file type. Alternatively, you can add headers manually:

| File type             | Comment style | Header                                                             |
| --------------------- | ------------- | ------------------------------------------------------------------ |
| Rust (`.rs`)          | `//`          | `// SPDX-FileCopyrightText: 2026 The Anytag Backend Authors`       |
| Markdown (`.md`)      | `<!-- -->`    | `<!-- SPDX-FileCopyrightText: 2026 The Anytag Backend Authors -->` |
| Shell (`.sh`)         | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Backend Authors`        |
| YAML (`.yml`/`.yaml`) | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Backend Authors`        |
| SQL (`.sql`)          | `--`          | `-- SPDX-FileCopyrightText: 2026 The Anytag Backend Authors`       |
| TOML (`.toml`)        | `#`           | `# SPDX-FileCopyrightText: 2026 The Anytag Backend Authors`        |

Each header must be followed by:

```text
// SPDX-License-Identifier: AGPL-3.0-only
```

## Checking Compliance Locally

Before committing, verify all files are properly annotated:

```bash
# Run the linter (through just — works without a mise-activated shell)
just reuse lint

# Alternatively, inside a mise-activated shell (mise extension terminal or
# `eval "$(mise activate)"` in your shell config), bare `reuse` works too:
reuse lint

# Expected output: "All files are compliant!"
```

## CI Enforcement

The CI pipeline (`.github/workflows/reuse.yml`) automatically runs `reuse lint` on every push and pull request. A non-compliant status will fail the build, so ensure all new files include proper SPDX headers before pushing.

## See Also

- [Development Guide](./DEVELOPMENT.md) — Development workflow (includes `reuse annotate` in Common Tasks)
- [Troubleshooting](./TROUBLESHOOTING.md) — CI/CD workflow overview
