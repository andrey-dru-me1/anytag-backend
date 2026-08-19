<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Troubleshooting

## Nix Issues

```bash
# Clear Nix cache if builds fail
nix-store --verify --check-contents

# Update Nix channels
nix-channel --update

# Enter shell with pure isolation
nix develop --pure
```

## Database Issues

```bash
# Check if database is running
docker compose ps

# View database logs
docker compose logs postgres

# Reset database (destructive!)
docker compose down -v
docker compose up -d postgres
diesel migration run
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

## Rust/Diesel Issues

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update

# Check diesel connection
diesel database reset
```

## See Also

- [Development Guide](./DEVELOPMENT.md) — Development workflow, common tasks, and CI/CD
- [Dependency Management](./DEPENDENCIES.md) — Adding and updating Rust and Nix dependencies
- [REUSE Compliance](./REUSE.md) — License management and SPDX headers
