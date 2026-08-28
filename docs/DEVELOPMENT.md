<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Development Guide for anytag-backend

## Recommended Setup: Docker + mise + just + rustup

The recommended way to develop this project is to pre-install four tools and
let [`mise.toml`](../mise.toml) and [`rust-toolchain.toml`](../rust-toolchain.toml)
manage the rest:

- **Docker** — runs PostgreSQL and SeaweedFS (local S3-compatible storage) containers
- **mise** — manages tool versions and environment variables; installs `diesel-cli`
  and `reuse`, and constructs `DATABASE_URL` for you
- **just** — command runner that wraps `cargo`, `diesel`, `test`, and `watch` with
  the mise-provided environment
- **rustup** — Rust toolchain manager; reads `rust-toolchain.toml` automatically

This replaces the previous Nix-based setup (still supported as an alternative —
see [Nix (Deprecated)](#nix-deprecated) below).

### Benefits

- **Working debugging on macOS** — the pinned Rust toolchain from
  [`rust-toolchain.toml`](../rust-toolchain.toml) (channel `1.98.0`) is installed via
  rustup and works out of the box with CodeLLDB/lldb, so breakpoints, Run/Debug
  buttons, and test debugging work reliably. Previously, cross-compiled Nix Rust
  toolchains often broke the debugger.
- **Lighter development environment** — no Nix store, no `nix develop` shell,
  no Nix daemon. Just native tools installed once on your machine.
- **Faster cold starts** — `nix develop` evaluation and store downloads are gone;
  `cargo` and `diesel` start immediately.
- **Native tooling** — `cargo`, `docker`, `just`, and `mise` are ordinary binaries
  on your `PATH`; IDE integration (rust-analyzer, VS Code tasks) works without a
  wrapper shell.
- **Deterministic pinning** — the Rust toolchain is pinned in
  [`rust-toolchain.toml`](../rust-toolchain.toml), and tool versions are pinned in
  [`mise.toml`](../mise.toml) (e.g. `diesel_cli = "2.3.12"`, `reuse = "6.2.0"`).

### 1. Install the Prerequisites

```bash
# Docker (macOS: Docker Desktop, Linux: Docker Engine) — https://www.docker.com
# mise — https://mise.jdx.dev
# just — https://just.systems
# rustup — https://rustup.rs
```

Install everything with your package manager, e.g. on macOS:

```bash
brew install --cask docker
brew install mise just rustup-init
```

> **Windows?** See [WINDOWS.md](./WINDOWS.md) for the WSL2 + mise flow.

### 2. Configure the Toolchain and Environment

rustup reads [`rust-toolchain.toml`](../rust-toolchain.toml) automatically the first
time you run `cargo`/`rustc` in the project, so no manual `rustup override` is needed.

mise reads [`mise.toml`](../mise.toml), which installs `diesel-cli` (via cargo) and
`reuse` (via pipx) and exports the environment. Prepare it with:

```bash
mise install
```

> **Activating the environment for bare commands.** Bare `cargo`, `diesel`, and
> `reuse` commands only see the mise-provided tools and the constructed
> `DATABASE_URL` inside a **mise-activated shell**. There are two ways to get one:
>
> 1. **Recommended:** use the integrated VS Code terminal with the
>    [mise VS Code extension](#running-commands-without-just-or-mise-prefixes)
>    (`mise.configureExtensionsAutomatically` enabled) — no shell changes needed.
> 2. Append `eval "$(mise activate)"` to the end of your shell config file
>    (e.g. `~/.zshrc` or `~/.bashrc`), then reload the shell.
>
> Until one of these is in place, use the `just` wrappers (`just diesel …`,
> `just cargo …`, `just test`), which always run through `mise x -- …` and need no
> activation.

### 3. Copy the Environment File

```bash
cp .env.example .env
```

The `.env` file holds the individual database components (`DB_TYPE`, `DB_HOST`,
`DB_PORT`, `DB_USER`, `DB_PASS`, `DB_NAME`) and the S3 settings. mise loads it via
`_.file = ".env"` in [`mise.toml`](../mise.toml) and constructs `DATABASE_URL` from
those components.

### 4. Start the Database and Object Storage

```bash
docker compose up -d
```

Starts PostgreSQL (port `DB_PORT`, default **5432**) and SeaweedFS (S3 access point
on `8333`).

### 5. Run Database Migrations

```bash
just diesel migration run
```

> The `just` recipe runs `diesel` through `mise x -- …`, so `DATABASE_URL` is always
> set and the command just works — no manual activation needed.
>
> The bare `diesel migration run` works too, but **only** inside a mise-activated
> shell (VS Code terminal with the mise extension, or `eval "$(mise activate)"` in
> your shell config) — see
> [Configure the Toolchain and Environment](#2-configure-the-toolchain-and-environment).

### 6. Build and Run

```bash
# Build (compile-only — does not need the environment)
cargo build

# Run with hot reload (needs DATABASE_URL)
just watch          # is: mise x -- cargo watch -x run
```

The server restarts automatically whenever you change a source file — just save,
then immediately test your endpoint with curl, httpie, or a REST client.

> Bare `cargo watch -x run` also works, but only inside a mise-activated shell —
> the server requires `DATABASE_URL`. The `just` wrappers (`just dev`,
> `just watch`, `just cargo build`) always load the environment for you.

### 7. Run Tests

```bash
just test
```

> Bare `cargo test` works too, but only in a mise-activated shell — the integration
> tests connect to PostgreSQL and need `DATABASE_URL`.

### Common Development Tasks

```bash
# Development server (hot reload) via just
just watch             # is: mise x -- cargo watch -x run

# Manual cargo/diesel/test via just
just cargo fmt
just cargo clippy
just test
just diesel migration generate migration_name
just diesel migration run
just diesel migration revert

# Test coverage (cargo-llvm-cov; install if you don't have it: cargo install cargo-llvm-cov)
mise x -- cargo llvm-cov                            # Print a coverage summary
mise x -- cargo llvm-cov --open                     # Open an HTML report
mise x -- cargo llvm-cov --lcov --output-path lcov.info   # LCOV report for editors
just cargo llvm-cov

# REUSE compliance (reuse is installed by mise via pipx:reuse)
mise x -- reuse lint
mise x -- reuse annotate --license AGPL-3.0-only --copyright "The Anytag Backend Authors" <file>
just reuse lint
```

Every `just` recipe and every `mise x -- ...` command runs through mise, so the
`DATABASE_URL` constructed by mise is available to `cargo`, `diesel`, and `reuse`.
Bare `cargo`/`reuse` invocations require a mise-activated shell (see
[Configure the Toolchain and Environment](#2-configure-the-toolchain-and-environment)).

### Running Commands Without `just` or `mise` Prefixes

If you use the official **mise VS Code extension**, enable
`mise.configureExtensionsAutomatically`. mise then:

- makes its tools (including `diesel`, `reuse`, and the pinned Rust toolchain)
  available directly in the integrated VS Code terminal — no `just` or
  `mise x --` prefix needed for `cargo`, `diesel`, `reuse`, or `rust-analyzer`;
- ensures rust-analyzer's **Run (test)** / **Debug (test)** code lenses work with
  the correct environment loaded (`DATABASE_URL` is constructed, `.env` is loaded).

This is the most convenient daily workflow for editor-driven development.

## Development Workflow

### 1. Start the Database and Object Storage

```bash
docker compose up -d
```

### 2. Run Database Migrations

```bash
just diesel migration run
```

### 3. Build the Project

```bash
just cargo build
```

### 4. Run the Development Server (Hot Reload)

```bash
just watch
```

### 5. Run Tests

```bash
just test
```

## Project Structure

The project follows a modular Rust web application architecture with clear separation of concerns. Key components include:

- **Application entry point** (`src/main.rs`) - Sets up the web server and routes
- **Application state** (`src/config.rs`) - Defines `AppState` (runtime resources: async database pool, S3 client) and `AppConfig` (immutable settings). `AppState::from_config()` auto-creates the media bucket on startup
- **Database models** (`src/models/`) - Define data structures and relationships
- **Request/response DTOs** (`src/dto/`) - Data transfer objects for API boundaries
- **HTTP handlers** (`src/handlers/`) - Process incoming requests and return responses
- **Database layer** - Connection management via `diesel-async` and deadpool (see `src/config.rs`)
- **Routing** (`src/router.rs`) - URL routing configuration

For the most current and detailed structure, please refer to the source code directly as the project evolves frequently.

## Environment Variables

The following environment variables are used:

### Individual Database Components (in `.env` file)

- `DB_USER=anytag` - Database username
- `DB_PASS=123456` - Database password
- `DB_NAME=anytag` - Database name
- `DB_PORT=5432` - Port for PostgreSQL (mapped from container port 5432). Default **5432**; configurable via `DB_PORT`.

### S3-Compatible Object Storage (in `.env` file)

The application stores media files in an S3-compatible object store (SeaweedFS locally, via Docker).

- `AWS_ACCESS_KEY_ID=anytag` - S3 access key (SeaweedFS credentials)
- `AWS_SECRET_ACCESS_KEY=change_me` - S3 secret key (SeaweedFS credentials)
- `S3_BUCKET=anytag-bucket` - Bucket used for media storage (auto-created on startup)
- `S3_BASE_URL=http://localhost:8333` - S3 endpoint URL (SeaweedFS S3 access point)

See [MEDIA.md](./MEDIA.md) for the full media storage architecture.

### Constructed Variables

- `DATABASE_URL` - Automatically constructed by [`.env`](../.env.example)-powered
  mise from the above components as:
  `${DB_TYPE}://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}`
- `RUST_BACKTRACE=1` (full backtraces on panic)
- `CARGO_TERM_COLOR=always` (colored output)

### How it works

1. The `.env` file contains individual database components
2. mise (via [`mise.toml`](../mise.toml)) automatically constructs `DATABASE_URL`
   from these components and exports it to every command started with
   `mise x -- ...` (which is what every `just` recipe does)
3. This avoids duplication - change a component in `.env` and `DATABASE_URL` updates automatically
4. `docker-compose.yaml` uses the individual components directly
5. `diesel-cli` and the Rust application use `DATABASE_URL`

## Nix (Deprecated)

The previous recommended setup used Nix (via `flake.nix` + direnv). It still works
and may be used as an alternative, but it is **deprecated** in favor of
Docker + mise + just + rustup (see above).

To use Nix:

```bash
cd anytag-backend
nix develop
# or with direnv: cp .envrc.example .envrc && direnv allow
```

The Nix shell provides the Rust toolchain, `diesel-cli`, PostgreSQL client, Docker
Compose, `just`, `reuse`, and other utilities, and constructs `DATABASE_URL` in its
`shellHook`. Note that on macOS the Nix-provided Rust toolchain historically breaks
the debugger (which is one of the reasons the mise-based setup is now recommended).

## Platform-Specific Notes

### macOS

- Docker Desktop required for database
- mise, just, and rustup install natively via Homebrew
- Debugging works out of the box with CodeLLDB + the rustup toolchain

### Linux

- Docker Engine or Docker Desktop
- May need to add user to docker group
- mise, just, and rustup install via your distribution's package manager

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
- [Dependency Management](./DEPENDENCIES.md) — Adding and updating Rust dependencies, and how mise manages tool versions
- [IDE Setup](./IDE_SETUP.md) — VS Code, Zed, and IntelliJ/CLion configuration
- [REUSE Compliance](./REUSE.md) — License management and SPDX headers (reuse runs via mise)
- [Git Workflow](./GIT_WORKFLOW.md) — Branch strategy, commit conventions, and PR guidelines
- [Windows Setup](./WINDOWS.md) — Windows-specific development setup with WSL2