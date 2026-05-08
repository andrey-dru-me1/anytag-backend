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
docker-compose ps

# View database logs
docker-compose logs db

# Reset database (destructive!)
docker-compose down -v
docker-compose up -d db
diesel migration run
```

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
