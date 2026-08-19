<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Build / Lint / Test Commands

- **Run a single test**: `cargo test test_name` — tests are co-located in `#[cfg(test)] mod tests` blocks within each source file
- **Single-file tests**: Tests live in `#[cfg(test)] mod tests { ... }` inside the source file (not in separate test files)
- **CI enforces strict clippy**: `cargo clippy -- -D warnings` — all warnings are errors
- **CI enforces formatting**: `cargo fmt -- --check` must pass
- **Migration commands**: `diesel migration run` / `diesel migration revert` / `diesel migration generate <name>`

## Code Style & Non-Obvious Conventions

- **Rust Edition 2024** — ensure your toolchain supports it (Nix provides it via `rust-overlay`)
- **All files MUST have SPDX headers**: Every file starts with `// SPDX-License-Identifier: AGPL-3.0-only` followed by `// Copyright (C) 2026 The Anytag Backend Authors`
- **Custom `bon::Builder` for error types**: Use `#[derive(bon::Builder)]` with `#[builder(derive(Clone))]` pattern from [`src/handlers/mod.rs`](src/handlers/mod.rs:36) (not hand-written constructors)
- **Unified error type `ApiError`**: All non-trivial handlers return `Result<_, ApiError>` — a `bon::Builder` struct carrying an `http_status`, a machine-readable `ApiErrorCode`, a `context` (logged), and an optional user-facing `message`. See [`src/handlers/mod.rs`](src/handlers/mod.rs:38). Simple handlers may still use `Result<_, (StatusCode, String)>` tuples.
- **Enum serialization via `strum`**: Error codes use `#[derive(AsRefStr, Display)]` with `#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]` — produces `WEAK_PASSWORD`, `DB_CONNECTION_ERROR`, `FILE_UPLOAD_ERROR`, `S3_STORAGE_ERROR`, etc. (`ApiErrorCode` enum in [`src/handlers/mod.rs`](src/handlers/mod.rs:22))
- **Password validation**: Must pass zxcvbn with `Score::Three` minimum (checked via `estimate.score() < Score::Three`)
- **Password hashing**: Argon2 with `rand_core::OsRng` salt — see [`src/handlers/users.rs`](src/handlers/users.rs:21)
- **Type aliases for IDs**: `Id = i32`, `UserId = Id`, `PostId = Id`, `TagId = Id`, `UserImageId = Id` in [`src/models/mod.rs`](src/models/mod.rs:5)
- **Model -> DTO conversion**: Via `impl From<Model> for Response`, pattern: [`src/dto/posts.rs`](src/dto/posts.rs:23)
- **Chrono `NaiveDateTime`** used for all timestamps; `.to_string()` produces `"1970-01-01 00:00:00"` format in DTOs
- **`#[allow(deprecated)]`** is intentionally used in tests for Chrono's `from_timestamp_opt`/`timestamp()` methods

## Architecture

- **Axum** with `State<Config>` extraction pattern for all handlers. Handlers pull a connection from the async pool via `config.db_pool.get().await?`
- **Async DB layer — no `src/db.rs`**: The `DbPool` alias is `Pool<AsyncPgConnection>` (diesel-async + deadpool) defined in [`src/config.rs`](src/config.rs:14). All queries use `diesel_async` traits (`AsyncConnection`, `RunQueryDsl`)
- **Config struct**: [`Config`](src/config.rs:17) carries `db_pool`, and (for media) `s3_client` (`aws_sdk_s3::Client`) and `s3_media_bucket`. Built by [`setup_config()`](src/config.rs:41), which also provisions the S3 bucket on startup
- **Media / S3 subsystem**: [`src/handlers/images.rs`](src/handlers/images.rs) implements image upload (`POST /api/v1/media/images`, multipart `file` field) and retrieval (`GET /api/v1/media/images/{image_name}`). Files are stored in an S3-compatible bucket keyed by content hash (`images/{sha256}.{ext}`); references live in `image_sources` / `user_images` tables
- **Database URL** is auto-constructed from `DB_USER`, `DB_PASS`, `DB_NAME`, `DB_PORT` env vars in Nix shellHook — local port defaults to **54321** (not 5432)
- **S3 env vars** (SeaweedFS locally): `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `S3_BUCKET`, `S3_BASE_URL` — see `.env.example` and [`docs/MEDIA.md`](docs/MEDIA.md)

## Git & PR Conventions

- **YouTrack ticket ID required** in every commit: `feat(ANY-1234): Subject`, `fix(AT-5678): Subject`, etc.
- **Branch naming**: `feature/ANY-1234-description`, `bugfix/ANY-5678-description`, `hotfix/ANY-9012-description`
- **PR title format**: `[Type] TICKET-ID: Brief description`
- **Merge strategy priority**: merge commit > rebase + merge commit

## Agent Skills

This project provides pre-built skills (in [`.agents/skills/`](.agents/skills/)) for recurring agentic tasks:

- **[`git-commit-message`](.agents/skills/git-commit-message/SKILL.md)** — Formulate commit messages with mandatory YouTrack ticket IDs
- **[`git-pull-request-message`](.agents/skills/git-pull-request-message/SKILL.md)** — Formulate PR titles/descriptions with template
- **[`reuse-compliance`](.agents/skills/reuse-compliance/SKILL.md)** — Add SPDX headers to new files

Skills are loaded automatically when the agent detects a matching task. The setup script at [`.agents/scripts/setup_agent_skills.sh`](.agents/scripts/setup_agent_skills.sh) creates symlinks from the agent's config dir (`.roo/`, `.cursor/`, etc.) to the canonical skill sources.
