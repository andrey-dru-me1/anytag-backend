<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Ask Mode — Project Documentation Rules (Non-Obvious Only)

- **Database URL is never hardcoded** — it's constructed by mise (`mise.toml`) from component env vars loaded from `.env`, not set directly in `.env`
- **`src/schema.rs` is auto-generated** by Diesel CLI — do not edit manually; run `diesel migration run` to regenerate after migration changes
- **Two error patterns exist**: tuple-based `(StatusCode, String)` for simple handlers and structured `ApiError` with `ApiErrorCode` enum for complex handlers — questioning which one to use is expected
- **`.envrc` is in `.gitignore`** (see [`.gitignore`](../../.gitignore:105)) but `.envrc.example` is tracked — copy the example, don't create from scratch
- **Tests are co-located** inside source files in `#[cfg(test)] mod tests` blocks — there is no `tests/` directory
- **mise + just is the primary dev environment** — bare `cargo`/`diesel` work only inside a mise-activated terminal (e.g. mise VS Code extension); both need `DATABASE_URL`, which mise constructs. Nix (`nix develop`) remains as a deprecated alternative
