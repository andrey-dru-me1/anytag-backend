<!--
SPDX-FileCopyrightText: 2026 The Anytag Backend Authors

SPDX-License-Identifier: AGPL-3.0-only
-->

# Plan: Replace verbose connection type with `DbConn` alias

## Goal

Replace `deadpool::managed::object::Object<AsyncDieselConnectionManager<AsyncPgConnection>>` in `src/handlers/images.rs:230` with a short type alias, mirroring the existing `DbPool` convention.

## Background

- `src/config.rs:14` defines `pub type DbPool = Pool<AsyncPgConnection>;` where `Pool` comes from `diesel_async::pooled_connection::deadpool`.
- `config.db_pool.get().await?` yields `deadpool::managed::object::Object<AsyncDieselConnectionManager<AsyncPgConnection>>`, which is the type spelled out at `src/handlers/images.rs:230`.
- The `Object` implements `Deref/DerefMut<Target = AsyncPgConnection>`, so the existing `insert_image(&mut AsyncPgConnection, ...)` call site keeps working unchanged.

## Changes

### 1. `src/config.rs`

Extend the import at line 9 and add a new alias after `DbPool` (line 14):

```rust
use diesel_async::{
    pg::AsyncPgConnection,
    pooled_connection::{
        AsyncDieselConnectionManager,
        deadpool::{Object, Pool},
    },
};

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConn = Object<AsyncDieselConnectionManager<AsyncPgConnection>>;
```

`diesel_async::pooled_connection::deadpool` re-exports the `deadpool` crate, so `Object` is importable from there (no new dependency needed).

### 2. `src/handlers/images.rs`

- Add `DbConn` to the existing crate import (line 19-25): `crate::config::DbConn`.
- Change `extract_image` signature at line 230:
  ```rust
  async fn extract_image(image_name: String, mut conn: DbConn) -> Result<Image, ApiError> {
  ```
- Remove the now-unneeded `mut` on the local at line 262 in `get_media` (`let mut conn = ...` is moved by value into `extract_image`, never mutated — avoids `unused_mut` clippy failure).

## Validation

- `cargo clippy -- -D warnings`
- `cargo fmt -- --check`
- `cargo test`

## Risks

None. Pure type-alias refactor; `Object` derefs to `AsyncPgConnection` so `insert_image` and `images.find(...).first(&mut conn)` compile unchanged.
