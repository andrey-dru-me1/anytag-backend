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
docker-compose up -d db
```

### 2. Run Database Migrations

```bash
diesel migration run
```

### 3. Build the Project

```bash
cargo build
```

### 4. Run Tests

```bash
cargo test
```

### 5. Common Development Tasks

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

# Build for release
cargo build --release
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

## Troubleshooting

### Nix Issues

```bash
# Clear Nix cache if builds fail
nix-store --verify --check-contents

# Update Nix channels
nix-channel --update

# Enter shell with pure isolation
nix develop --pure
```

### Database Issues

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

### Rust/Diesel Issues

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update

# Check diesel connection
diesel database reset
```

## IDE Integration

### VS Code

Recommended extensions for Rust development with Nix:

- **rust-analyzer** – Rust language support
- **CodeLLDB** – Debugging support
- **Nix IDE** – Nix language support - syntax highlighting, formatting, and error reporting
- **direnv** – Automatic environment loading
- **DockerDX** – Docker integration
- **Even Better TOML** – TOML file support
- **YAML** – YAML file support

Open the project - VS Code should detect the Nix environment automatically.

### Zed

Zed has built-in Rust support via rust-analyzer. For Nix development:

1. Install the **nix** extension from the extensions panel
2. Open the project - Zed will detect the Nix environment
3. Enable **direnv** support in settings if using direnv

### IntelliJ/CLion

1. Install "Rust" plugin
2. Open project - may need to configure custom toolchain
3. Set environment variables in run configurations

## Adding New Dependencies

### Rust Dependencies

Edit `Cargo.toml` and run:

```bash
cargo build  # Updates Cargo.lock
```

### Nix Dependencies

Edit `flake.nix` and re-enter shell:

```bash
nix develop
```

If you're using direnv, reload the environment after editing `flake.nix`:

```bash
direnv reload
```

## CI/CD

The project includes GitHub Actions workflow (`.github/workflows/ci.yml`) that:

1. Tests on Ubuntu, macOS, and Windows
2. Runs all migrations
3. Builds and tests the project
4. Checks formatting and linting

## Contributing

1. Ensure all tests pass: `cargo test`
2. Format code: `cargo fmt`
3. Check linting: `cargo clippy`
4. Update documentation if needed
5. Create pull request

## Need Help?

- Check the [WINDOWS.md](./WINDOWS.md) for Windows-specific issues
- Review Nix documentation: <https://nixos.org/learn/>
- Diesel documentation: <https://diesel.rs/guides/>
- Create an issue in the repository
