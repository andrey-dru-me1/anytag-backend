<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Code Mode — Project Coding Rules (Non-Obvious Only)

- **Use `bon::Builder` for error structs**, not hand-written constructors — see [`src/handlers/mod.rs`](../../src/handlers/mod.rs:30)
- **Simple handlers** return `Result<_, (StatusCode, String)>`; **complex handlers** (e.g., users) return `Result<_, ApiError>` — both coexist in the same project
- **Error codes** must be added to the [`ApiErrorCode`](../../src/handlers/mod.rs:19) enum with `#[derive(AsRefStr, Display)]` and `#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]`
- **All files MUST have SPDX headers** — `// SPDX-License-Identifier: AGPL-3.0-only` + `// Copyright (C) 2026 The Anytag Backend Authors`
- **Password validation** requires `zxcvbn` with `Score::Three` minimum (see [`src/handlers/users.rs`](../../src/handlers/users.rs:36))
- **Password hashing** must use Argon2 with `rand_core::OsRng` salt — not bcrypt, not SHA
- **Model -> DTO conversion** uses `impl From<Model> for Response` (not `IntoResponse`, not manual mapping)
- **`#[allow(deprecated)]`** is intentional in tests for `from_timestamp_opt`/`timestamp()` — do not remove
- **Type aliases**: `Id = i32`, `UserId = Id`, `PostId = Id`, `TagId = Id` — always use these, not raw `i32`

## Available Skills

Project skills at [`.agents/skills/`](../../.agents/skills/) provide agent instructions for recurring tasks:

- `reuse-compliance` — adding SPDX headers to new files
- `git-commit-message` — formatted commit messages with YouTrack IDs
- `git-pull-request-message` — PR title/description templates

Run [`.agents/scripts/setup_agent_skills.sh`](../../.agents/scripts/setup_agent_skills.sh) to symlink skills into your agent config dir.
