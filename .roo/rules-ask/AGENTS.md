<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
SPDX-License-Identifier: AGPL-3.0-only
-->

# Ask Mode — Project Documentation Rules (Non-Obvious Only)

- **Database URL is never hardcoded** — it's constructed from component env vars in the Nix shellHook (see [`flake.nix`](../../flake.nix:65)), not set directly in `.env`
- **`src/schema.rs` is auto-generated** by Diesel CLI — do not edit manually; run `diesel migration run` to regenerate after migration changes
- **Two error patterns exist**: tuple-based `(StatusCode, String)` for simple handlers and structured `ApiError` with `ApiErrorCode` enum for complex handlers — questioning which one to use is expected
- **`.envrc` is in `.gitignore`** (see [`.gitignore`](../../.gitignore:105)) but `.envrc.example` is tracked — copy the example, don't create from scratch
- **Tests are co-located** inside source files in `#[cfg(test)] mod tests` blocks — there is no `tests/` directory
- **Nix is the primary dev environment** — `cargo build` without `nix develop` may fail due to missing `libpq` or wrong toolchain
