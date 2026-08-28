<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Troubleshooting

## Environment / mise Issues

```bash
# Reinstall/update mise-managed tools (diesel-cli, reuse)
mise install

# Resolve the mise environment (loads .env, constructs DATABASE_URL)
mise x -- env

# If your terminal is not mise-activated, use the just wrappers:
just cargo build
just diesel migration run
```

## Rust/Diesel Issues

```bash
# Ensure the pinned toolchain is installed (rust-toolchain.toml: 1.98.0)
rustup toolchain install 1.98.0

# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update

# Check diesel connection
just diesel database reset
```

Common issues:

- **"diesel command not found"**: install via `mise install`, or use `just diesel ...`
- **"DATABASE_URL is not set" / "Failed to create database pool"**: verify `.env` exists
  (`cp .env.example .env`) and run inside mise (`mise x -- ...`, `just ...`, or a
  mise-activated VS Code terminal)
- **rust-analyzer run/debug buttons fail with "DATABASE_URL is not set"**: the test
  was launched outside the mise environment — enable the mise VS Code extension with
  `mise.configureExtensionsAutomatically` (see [`IDE_SETUP.md`](./IDE_SETUP.md))

## Database Issues

```bash
# Check if database is running
docker compose ps

# View database logs
docker compose logs postgres

# Reset database (destructive!)
docker compose down -v
docker compose up -d postgres
just diesel migration run   # through just so DATABASE_URL is set
```

## S3 / SeaweedFS Issues

```bash
# Check if the SeaweedFS container is running
docker compose ps

# View SeaweedFS logs
docker compose logs seaweed

# Reset SeaweedFS storage (destructive — wipes uploaded media)
docker compose down -v
docker compose up -d
```

Common issues:

- **"failed to create new bucket" / S3 errors on startup**: Ensure the SeaweedFS container is up before starting the app. The app auto-creates `S3_BUCKET` on startup — check `S3_BUCKET`, `AWS_ACCESS_KEY_ID`, and `AWS_SECRET_ACCESS_KEY` in `.env` match the values used by the SeaweedFS container (the S3 access point listens on port `8333`).
- **`S3_BASE_URL` not set**: `S3_BASE_URL` is required by the app (the local SeaweedFS S3 endpoint is `http://localhost:8333`). Verify it is present in `.env`.
- **403 / signature errors**: SeaweedFS and the app must share the same `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` (defined in `docker-compose.yaml`).

## Nix Issues (Deprecated Environment)

Only relevant if you use the deprecated Nix setup:

```bash
# Clear Nix cache if builds fail
nix-store --verify --check-contents

# Update Nix channels
nix-channel --update

# Enter shell with pure isolation
nix develop --pure
```

## See Also

- [Development Guide](./DEVELOPMENT.md) — Development workflow, common tasks, and CI/CD
- [Dependency Management](./DEPENDENCIES.md) — Adding and updating Rust dependencies and mise-managed tools
- [REUSE Compliance](./REUSE.md) — License management and SPDX headers
