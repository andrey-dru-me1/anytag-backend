<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Development Guide for anytag-backend

## Quick Start

### 1. Install Nix (Determinate Systems installer)

```bash
# One-time installation (all platforms)
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

### 2. Enter the Development Environment

```bash
cd anytag-backend
nix develop
# or if you prefer the traditional shell: nix-shell --flake .
```

This provides a complete development environment with:

- Rust toolchain (stable) with rustfmt, clippy, rust-analyzer
- diesel-cli 2.3.0 (matches your Cargo.toml)
- PostgreSQL client (psql, libpq) version 16
- Docker Compose for database container
- Git and development utilities

### 3. Optional: Automatic Environment with direnv

For automatic environment loading when you `cd` into the project:

```bash
# Install direnv
# macOS:
brew install direnv
# Linux:
sudo apt-get install direnv

# Hook direnv into your shell (follow post-install instructions)

# Copy the direnv example to create your local .envrc
cp .envrc.example .envrc

# Allow direnv in this project
cd anytag-backend
direnv allow
```

Now the environment loads automatically whenever you enter the project directory!

**Note:** `.envrc` is in `.gitignore` to keep local configurations out of version control. The template file `.envrc.template` contains the base configuration that should be copied to `.envrc` for local use.

## Development Workflow

### 1. Start the Database

```bash
docker compose up -d db
```

### 2. Run Database Migrations

```bash
diesel migration run
```

### 3. Build the Project

```bash
cargo build
```

### 4. Run the Development Server (Hot Reload)

```bash
cargo watch -x run
```

The server restarts automatically whenever you change a source file —
just save, then immediately test your endpoint with curl, httpie, or a
REST client. No manual `cargo run` needed after every edit.

### 5. Run Tests

```bash
cargo test
```

### 6. Common Development Tasks

```bash
# Create new migration
diesel migration generate migration_name

# Revert last migration
diesel migration revert

# Format code
cargo fmt

# Lint code
cargo clippy

# Run specific test
cargo test test_name

# Run tests automatically on file changes
cargo watch -x test

# Build for release
cargo build --release

# Annotate new files with SPDX headers (see REUSE Compliance)
reuse annotate --license AGPL-3.0-only --copyright "The Anytag Backend Authors" <file>
```

## Project Structure

The project follows a modular Rust web application architecture with clear separation of concerns. Key components include:

- **Application entry point** (`src/main.rs`) - Sets up the web server and routes
- **Database models** (`src/models/`) - Define data structures and relationships
- **Request/response DTOs** (`src/dto/`) - Data transfer objects for API boundaries
- **HTTP handlers** (`src/handlers/`) - Process incoming requests and return responses
- **Database layer** (`src/db.rs`) - Connection management and query utilities
- **Routing** (`src/router.rs`) - URL routing configuration

For the most current and detailed structure, please refer to the source code directly as the project evolves frequently.

## Environment Variables

The following environment variables are used:

### Individual Database Components (in `.env` file)

- `POSTGRES_USER=anytag` - Database username
- `POSTGRES_PASSWORD=123456` - Database password
- `POSTGRES_DB=anytag` - Database name
- `DB_PORT=54321` - Port for PostgreSQL (mapped from container port 5432)

### Constructed Variables

- `DATABASE_URL` - Automatically constructed from the above components as: `postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost:${DB_PORT}/${POSTGRES_DB}`
- `RUST_BACKTRACE=1` (full backtraces on panic)
- `CARGO_TERM_COLOR=always` (colored output)

### How it works

1. The `.env` file contains individual database components
2. The Nix shell (`flake.nix`) automatically constructs `DATABASE_URL` from these components
3. This avoids duplication - change a component in `.env` and `DATABASE_URL` updates automatically
4. `docker-compose.yaml` uses the individual components directly
5. `diesel-cli` and the Rust application use `DATABASE_URL`

## Platform-Specific Notes

### macOS

- Nix works natively
- Docker Desktop required for database
- No additional setup needed

### Linux

- Nix works natively
- Docker Engine or Docker Desktop
- May need to add user to docker group

### Windows

See [WINDOWS.md](./WINDOWS.md) for detailed Windows/WSL2 setup.

## CI/CD

The project includes two GitHub Actions workflows that run on every push and pull request to `master` and `develop`:

- [`.github/workflows/ci.yml`](.github/workflows/ci.yml) — Builds, tests, lints, and checks formatting
- [`.github/workflows/reuse.yml`](.github/workflows/reuse.yml) — REUSE compliance check

## Contributing

1. Ensure all tests pass: `cargo test`
2. Format code: `cargo fmt`
3. Check linting: `cargo clippy`
4. Update documentation if needed
5. Create pull request

## See Also

- [Troubleshooting](./TROUBLESHOOTING.md) — Common issues and solutions
- [Dependency Management](./DEPENDENCIES.md) — Adding and updating Rust and Nix dependencies
- [IDE Setup](./IDE_SETUP.md) — VS Code, Zed, and IntelliJ/CLion configuration
- [REUSE Compliance](./REUSE.md) — License management and SPDX headers
- [Git Workflow](./GIT_WORKFLOW.md) — Branch strategy, commit conventions, and PR guidelines
- [Windows Setup](./WINDOWS.md) — Windows-specific development setup with WSL2
