<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# IDE Setup for anytag-backend

## VS Code

Recommended extensions for Rust development with Nix:

- **rust-analyzer** – Rust language support
- **CodeLLDB** – Debugging support
- **Nix IDE** – Nix language support - syntax highlighting, formatting, and error reporting
- **direnv** – Automatic environment loading
- **DockerDX** – Docker integration
- **Even Better TOML** – TOML file support
- **YAML** – YAML file support
- **Coverage Gutters** – Test coverage highlighting in the editor

Open the project — VS Code should detect the Nix environment automatically.

### Viewing Test Coverage

Test coverage is measured with [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) (included in the Nix environment). The **Coverage Gutters** extension (listed in [`.vscode/extensions.json`](../.vscode/extensions.json)) renders the coverage directly in the editor.

1. Generate an LCOV report (the format Coverage Gutters reads):

   ```bash
   cargo llvm-cov --lcov --output-path lcov.info
   ```

2. In VS Code, run **Coverage Gutters: Watch** (or **Coverage Gutters: Display Coverage**) from the command palette.

3. Alternatively, view coverage without the editor:

   ```bash
   cargo llvm-cov          # Coverage summary in the terminal
   cargo llvm-cov --open   # HTML report in the browser
   ```

`lcov.info` is a generated artifact and is listed in `.gitignore`, so it is never committed.

## Zed

Zed has built-in Rust support via rust-analyzer. For Nix development:

1. Install the **nix** extension from the extensions panel
2. Open the project — Zed will detect the Nix environment
3. Enable **direnv** support in settings if using direnv

## IntelliJ/CLion

1. Install "Rust" plugin
2. Open project — may need to configure custom toolchain
3. Set environment variables in run configurations

## See Also

- [Development Guide](./DEVELOPMENT.md) — Development workflow and common tasks
- [Troubleshooting](./TROUBLESHOOTING.md) — Common issues and solutions
