<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Architect Mode — Project Architecture Rules (Non-Obvious Only)

- **Two error patterns coexist by design**: tuple `Result<_, (StatusCode, String)>` for simple read-only handlers, and structured `ApiError` with `ApiErrorCode` enum for complex mutation handlers — new handlers should match the pattern of their domain (see [`src/handlers/mod.rs`](../../src/handlers/mod.rs:19))
- **Model -> DTO conversion** uses `impl From<Model> for Response` in DTO modules (see [`src/dto/posts.rs`](../../src/dto/posts.rs:23)) — this is the only conversion pattern; no manual mapping or IntoResponse on models
- **Error structs use `bon::Builder`** with `#[builder(derive(Clone))]` — not hand-written constructors or `derive_builder`
- **No authentication middleware exists yet** — the current architecture is pre-auth; handlers read `State<DbPool>` directly
- **Database port 5432** is the default (configurable via `DB_PORT`) — matches `docker-compose.yaml` and `.env.example`
- **The Rust toolchain comes from rustup** via [`rust-toolchain.toml`](../../rust-toolchain.toml) (`channel = "1.98.0"`) — `diesel-cli` and `reuse` are provided by mise (`mise.toml`), not Nix
- **Merge strategy** prefers merge commits over squashing — see [`docs/GIT_WORKFLOW.md`](../../docs/GIT_WORKFLOW.md:209)
