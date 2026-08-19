<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Fix `DATABASE_URL` composition (dotenvy does not interpolate `${VAR}`)

## Problem

`DATABASE_URL` is stored in `.env` as an **unexpanded** template:
`DATABASE_URL=${DB_TYPE}://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}`.

`dotenvy` does **not** expand `${VAR}` references, so Rust apps and diesel read
the literal template string, not a real URL. Only direnv's `dotenv .env`
(which uses shell interpolation) made it work locally — silently broken for
anyone running outside direnv, and always broken in CI (no `.env`, GitHub
Actions does not expand `${VAR}` in workflow `env:`).

`src/config.rs` was already fixed to compose the URL in Rust from `DB_*` vars,
but four other spots still trust a pre-built `DATABASE_URL`.

## Remaining work

### 1. `tests/common/mod.rs` — compose like `setup_database()`
`test_db_pool()` (L17-27) reads `DATABASE_URL` only. Mirror the `src/config.rs`
logic: if `DATABASE_URL` is missing, compose from `DB_TYPE/DB_HOST/DB_PORT/DB_USER/DB_PASS/DB_NAME`.

### 2. `.github/workflows/ci.yml` — fix/remove broken `DATABASE_URL` line
L26 `DATABASE_URL: ${DB_TYPE}://...` is not expanded by GHA. Since Rust (runtime
+ tests) now composes from `DB_*`, the cleanest fix:
- Remove the `DATABASE_URL` line from the workflow `env:` entirely; keep the six
  `DB_*` vars.
- diesel-cli still needs `DATABASE_URL` for `migration run`.
  **Required:** add a step-level `env:` that sets `DATABASE_URL` using GHA context
  expressions (`${{ env.DB_USER }}` etc.) — valid only at the **step** level, not
  workflow `env:`. Example:
  ```yaml
  - name: Run migrations
    env:
      DATABASE_URL: postgres://${{ env.DB_USER }}:${{ env.DB_PASS }}@${{ env.DB_HOST }}:${{ env.DB_PORT }}/${{ env.DB_NAME }}
    run: nix develop --command diesel migration run
  ```
  (In the `test-nix` job, `DB_HOST`/`DB_PORT` default to `localhost:5432` and the
  Postgres service maps to them, so this yields a working URL.)

### 3. `.env` template cleanup — remove the landmine
`setup_database()` only composes when `DATABASE_URL` is **absent**. The current
`.env` has `DATABASE_URL` set to the literal template, so `env::var` returns it
and the composition fallback never fires. To make the Rust composition reachable
(and harmless outside direnv):
- Delete the `DATABASE_URL=...` line from `.env` and `.env.example`.
  Alternatively, if keeping `.env` for diesel-cli local use, ensure it holds a
  **real** expanded URL — but deleting and relying on `DB_*` composition is
  simpler and single-source-of-truth.
- Caveat: diesel-cli run locally also needs `DATABASE_URL`. Either keep a real
  expanded URL in `.env`/direnv, or export it from direnv. Document the expectation
  (see #5).

### 4. Guard against a pre-set template value
Hardening (defensive): in `setup_database()`/`test_db_pool()`, treat a
`DATABASE_URL` that contains `${` as "not a URL" and fall through to composition.
Prevents recurrence if a template ever gets committed again. Optional but cheap.

### 5. Update stale docs + flake comment
- `docs/DEVELOPMENT.md:168-172` claims "Nix shell automatically constructs
  `DATABASE_URL`" — this shellHook was removed earlier in the branch. Update to
  describe the new Rust composition (and `DB_*` vars as source of truth).
- `flake.nix` shellHook no longer prints the DB-construction warning; no change
  needed but confirm nothing references `DATABASE_URL` construction.

## Validation

- `cargo build` and `cargo test` (with `DB_*` set, no `DATABASE_URL`) pass in a
  shell **without** direnv.
- `diesel migration run` succeeds in CI via the new step-level `DATABASE_URL`.
- Inspect CI output: `diesel --version`/migration step must use a real
  `postgres://postgres:change_me@localhost:5432/anytag_test` URL, not a literal
  `${...}` template.
- `cargo clippy -- -D warnings` and `cargo fmt -- --check` clean.

## Open questions
- Keep a real (expanded) `DATABASE_URL` in `.env` for diesel-cli, or require
  direnv/export? Recommended: keep `.env` with expanded URL OR remove and rely
  on `use flake`+a real URL — decide before implementing #3.
