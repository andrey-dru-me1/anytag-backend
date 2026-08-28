<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Debug Mode — Project Debug Rules (Non-Obvious Only)

- **Database port**: local PostgreSQL runs on **5432** by default (configurable via `DB_PORT`) — see [`docker-compose.yaml`](../../docker-compose.yaml:14) and `.env.example`
- **DATABASE_URL is auto-constructed** by mise (`mise.toml`) from `DB_TYPE`, `DB_HOST`, `DB_PORT`, `DB_USER`, `DB_PASS`, `DB_NAME` loaded from `.env` — if one of these is missing, `DATABASE_URL` won't be set and connections will fail silently. Run commands via `just ...` (which wraps `mise x -- ...`) or from a mise-activated terminal
- **Connection pool errors** return `(StatusCode, String)` tuples, but the `ApiError` type wraps them via [`ApiError::from_db_conn_err`](../../src/handlers/mod.rs:42) — check which pattern the handler uses
- **Tests need a running PostgreSQL** unless they're pure compilation tests; `#[cfg(test)]` blocks in router and handlers use fake URLs with short timeouts to avoid hanging during unit tests without a DB
- **`RUST_BACKTRACE=1`** is exported by the mise env (`mise.toml`) — if you run outside a mise-activated terminal, set it manually
- **SPDX violations fail CI** via the REUSE workflow — run `reuse lint` locally to check
