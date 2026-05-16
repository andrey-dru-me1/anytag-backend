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

Open the project — VS Code should detect the Nix environment automatically.

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
