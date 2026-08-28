<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Anytag Backend

![logo](assets/main_logo.svg)

A Rust backend service for a social tagging application built with Diesel ORM and PostgreSQL.

## Project Overview

Anytag is a social platform where users can:

- Create posts with text content
- Create and manage tags
- Subscribe to tags and other users
- Control visibility of tags to specific users
- Upload and retrieve media images (stored in an S3-compatible object store)

## Tech Stack

- **Language**: Rust 2024 Edition
- **Framework**: Axum with Diesel ORM
- **Database**: PostgreSQL 18.3
- **Object Storage**: S3-compatible (local SeaweedFS via Docker)
- **Containerization**: Docker & Docker Compose
- **ORM**: Diesel 2.3 with diesel-async (async PostgreSQL support)
- **Environment Management**: docker + mise + just + rustup (Nix kept as deprecated alternative)

## Quick Start

### 1. Prerequisites

Install these tools once on your machine:

- [Docker](https://www.docker.com) — runs PostgreSQL and SeaweedFS (local S3-compatible storage)
- [mise](https://mise.jdx.dev) — manages tool versions and environment variables (installs `diesel-cli` and `reuse`, constructs `DATABASE_URL`)
- [just](https://just.systems) — command runner wrapping `cargo`, `diesel`, `test`, and `watch`
- [rustup](https://rustup.rs/) — Rust toolchain manager; reads `rust-toolchain.toml` automatically

### 2. Setup (Recommended: mise + just)

```bash
# Copy environment configuration
cp .env.example .env

# Install mise-managed tools (diesel-cli, reuse) and export the environment
mise install

# Start the database and local S3-compatible storage (SeaweedFS)
docker compose up -d

# Setup database with migrations (DATABASE_URL is constructed automatically by mise)
just diesel database setup

# Build the project
just cargo build

# Run the development server with hot reload (restarts on file changes)
just watch

# Run tests
just test
```

> Every `just` recipe runs through `mise x -- ...`, so `cargo`, `diesel`, and
> `reuse` always see the correct environment (`DATABASE_URL` constructed from `.env`).
> You can equally run `cargo`, `diesel`, `reuse` directly — but only inside a
> mise-activated shell/terminal.

### 3. Nix (Deprecated Alternative)

The previous Nix-based setup still works but is deprecated:

```bash
nix develop
# or with direnv: cp .envrc.example .envrc && direnv allow
```

### 4. IDE Notes

Install the **mise VS Code extension** and enable `mise.configureExtensionsAutomatically`.
mise then makes `cargo`, `diesel`, `reuse`, and the pinned Rust toolchain available
directly in the VS Code terminal — no `just`/`mise x --` prefix needed — and
rust-analyzer's **Run (test) / Debug (test)** buttons work with the environment
loaded (`DATABASE_URL` constructed). See [docs/IDE_SETUP.md](docs/IDE_SETUP.md).

## Project Structure

The project follows a standard Rust web application structure with separate modules for models, DTOs, handlers, and database operations. For the most up-to-date structure, refer to the source code directly.

## Database

The application uses 9 main tables:

1. **users** - User accounts with authentication
2. **posts** - User-created posts
3. **tags** - User-created tags with privacy settings
4. **post_tags** - Many-to-many relationship between posts and tags
5. **user_tag_subscriptions** - Users subscribing to tags
6. **user_user_subscriptions** - Users following other users
7. **tag_user_visibility** - Custom visibility settings for tags
8. **image_sources** - Deduplicated image content metadata hosted in S3
9. **user_images** - User-uploaded images referencing an `image_sources` entry

For details on the media/image subsystem (endpoints, storage layout, and SeaweedFS), see [docs/MEDIA.md](docs/MEDIA.md).

## Common Commands

> **Note on bare commands.** Bare `cargo`/`diesel` commands only see the mise-provided
> tools and the constructed `DATABASE_URL` inside a **mise-activated shell** — either the
> integrated VS Code terminal with the mise extension (`mise.configureExtensionsAutomatically`)
> or after appending `eval "$(mise activate)"` to your shell config. To be safe without
> activation, prefix with `just` (the `just` wrappers run through `mise x -- …`).

```bash
# Development
just cargo check          # Check for compilation errors
just cargo test           # Run tests
just cargo fmt            # Format code
just cargo clippy         # Lint code
just watch                # Development server with hot reload (restarts on save)
just cargo watch -x test  # Run tests automatically on file changes

# Test coverage
just cargo llvm-cov --lcov --output-path lcov.info  # Generate an LCOV report for editors/IDEs
just cargo llvm-cov --open                           # Open an HTML coverage report in the browser
just cargo llvm-cov                                  # Print a coverage summary to the terminal

# Database
just diesel migration run    # Run pending migrations
just diesel migration revert # Revert last migration
just diesel migration list   # List all migrations

# Docker
docker compose up -d        # Start database and SeaweedFS (S3-compatible) storage
docker compose down -v      # Stop and remove containers with volumes
```

## Documentation

Detailed documentation is available in the `docs/` directory:

- **[Development Guide](docs/DEVELOPMENT.md)** - Development workflow, project structure, and environment setup
- **[Troubleshooting](docs/TROUBLESHOOTING.md)** - Common issues and CI/CD overview
- **[Dependency Management](docs/DEPENDENCIES.md)** - Adding and updating Rust dependencies, how mise manages tool versions
- **[IDE Setup](docs/IDE_SETUP.md)** - VS Code, Zed, and IntelliJ/CLion configuration
- **[REUSE Compliance](docs/REUSE.md)** - License management and SPDX headers (reuse runs via mise)
- **[Git Workflow](docs/GIT_WORKFLOW.md)** - Branch strategy, commit conventions, and PR guidelines
- **[Windows Setup](docs/WINDOWS.md)** - Windows-specific development setup with WSL2
- **[Media & S3 Storage](docs/MEDIA.md)** - Image upload/retrieval endpoints and object storage layout
- **Database Schema** - See `migrations/` directory for SQL definitions

## Troubleshooting

Common issues and solutions:

- **"Connection refused"**: Ensure Docker is running and database container is up
- **"diesel command not found"**: Run `mise install` first, or use the `just diesel ...` wrappers
- **"DATABASE_URL is not set"**: Ensure `.env` exists (`cp .env.example .env`) and you run the command inside mise (`mise x -- ...`, `just ...`, or a mise-activated VS Code terminal)
- **"ld: library 'pq' not found"**: Install PostgreSQL development libraries for your platform

For detailed troubleshooting, see [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md).

## Contributing

Please read our [Git Workflow Guide](docs/GIT_WORKFLOW.md) for detailed guidelines on branch strategy, commit conventions, and pull request process.

Basic workflow:

1. Create a new branch for your feature (see naming conventions in Git workflow)
2. Add migrations for database changes
3. Update models if schema changes
4. Write tests for new functionality
5. Ensure `cargo check` and `cargo test` pass
6. Submit a pull request following the PR guidelines

## License

This project is licensed under the GNU Affero General Public License v3 (AGPLv3).

See the [LICENSE](LICENSE) file for the full license text.

### Key Points of AGPLv3

- **Copyleft**: Modifications must be released under the same license
- **Network Use**: Requires source code availability for network/server software
- **Commercial Use**: Permitted with compliance to license terms

### REUSE Compliance

This project follows the [REUSE Specification](https://reuse.software/) for license and copyright annotations. All files include SPDX headers — see the [REUSE Compliance Guide](docs/REUSE.md) for details on adding headers and running the linter.

For more details about the AGPLv3 license, visit:

- [GNU AGPLv3 License](https://www.gnu.org/licenses/agpl-3.0.html)
- [AGPLv3 FAQ](https://www.gnu.org/licenses/agpl-faq.html)
