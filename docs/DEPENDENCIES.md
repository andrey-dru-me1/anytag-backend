<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Dependency Management

## Adding Rust Dependencies

Use `cargo add` to add new dependencies:

```bash
# Add a production dependency
cargo add <crate_name>

# Add a development dependency
cargo add --dev <crate_name>

# Add with specific features
cargo add <crate_name> --features feature1,feature2
```

This automatically updates both `Cargo.toml` and `Cargo.lock`. Alternatively, you can manually edit `Cargo.toml` and then run `cargo build` to update the lockfile.

## Updating Rust Dependencies

```bash
# Update all dependencies within their version constraints
cargo update

# Update a specific crate to the latest compatible version
cargo update <crate_name>

# Check for newer versions (without updating)
cargo outdated
```

## Adding Nix Dependencies

Edit `flake.nix` and re-enter the shell:

```bash
nix develop
```

If you're using direnv, reload the environment after editing `flake.nix`:

```bash
direnv reload
```

## See Also

- [Development Guide](./DEVELOPMENT.md) — Development workflow and common tasks
- [Troubleshooting](./TROUBLESHOOTING.md) — Common issues and solutions
