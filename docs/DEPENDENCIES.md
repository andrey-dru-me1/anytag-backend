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

## Managing Tool Versions with mise

Tool versions are pinned in [`mise.toml`](../mise.toml) (e.g. `diesel_cli = "2.3.12"`,
`reuse = "6.2.0"`). After changing `mise.toml`, install/update the tools:

```bash
mise install
```

The Rust toolchain is pinned separately in [`rust-toolchain.toml`](../rust-toolchain.toml)
(`channel = "1.98.0"`) and installed automatically by rustup on first `cargo`/`rustc`
invocation. To pre-install it:

```bash
rustup toolchain install 1.98.0
```

### Run tools through just (recommended)

Every [`just`](../Justfile) recipe runs through `mise x -- ...`, so it always uses the
pinned tools and the constructed `DATABASE_URL`:

```bash
just cargo build
just cargo clippy
just test
just diesel migration run
just watch
```

## Adding Nix Dependencies (Deprecated)

Only relevant if you still use the deprecated Nix environment. Edit `flake.nix` and
re-enter the shell:

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
