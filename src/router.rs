// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use crate::db::DbPool;
use crate::handlers;
use axum::{
    Router,
    routing::{get, post},
};

/// Create and configure the Axum router
pub fn create_router(pool: DbPool) -> Router {
    let api_v1 = Router::new()
        .route("/", get(handlers::health_check))
        .route("/health", get(handlers::health_check))
        .route("/posts", get(handlers::list_posts))
        .route("/tags", get(handlers::list_tags))
        .route("/users", post(handlers::create_user))
        .route("/auth/login", post(handlers::login_user));

    Router::new().nest("/api/v1", api_v1).with_state(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time check that `create_router` accepts a `DbPool`.
    #[test]
    fn test_create_router_type_check() {
        let _: fn(DbPool) -> Router = create_router;
    }
}
