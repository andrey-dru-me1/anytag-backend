<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Debug Mode — Project Debug Rules (Non-Obvious Only)

- **Database port differs from default**: local PostgreSQL runs on **54321** (not 5432) — see [`docker-compose.yaml`](../../docker-compose.yaml:14)
- **DATABASE_URL is auto-constructed** from `DB_TYPE`, `DB_HOST`, `DB_PORT`, `DB_USER`, `DB_PASS`, `DB_NAME` in the Nix shellHook — if one of these is missing, `DATABASE_URL` won't be set and connections will fail silently
- **Connection pool errors** return `(StatusCode, String)` tuples, but the `ApiError` type wraps them via [`ApiError::from_db_conn_err`](../../src/handlers/mod.rs:42) — check which pattern the handler uses
- **Tests need a running PostgreSQL** unless they're pure compilation tests; `#[cfg(test)]` blocks in router and handlers use fake URLs with short timeouts to avoid hanging during unit tests without a DB
- **`RUST_BACKTRACE=1`** is set in the Nix devShell — if running outside Nix, set it manually
- **SPDX violations fail CI** via the REUSE workflow — run `reuse lint` locally to check
