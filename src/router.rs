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
    use diesel_async::pg::AsyncPgConnection;
    use diesel_async::pooled_connection::deadpool::Pool;

    #[test]
    fn test_create_router_with_mock_pool() {
        // Test that create_router compiles and accepts a DbPool
        // Verify function signature for the async connection pool.
        let _func: fn(Pool<AsyncPgConnection>) -> Router = create_router;
        assert!(true);
    }

    #[test]
    fn test_router_function_exists() {
        // Simple test to verify the function signature
        // This is a compilation test more than a runtime test
        let _ = create_router as fn(Pool<AsyncPgConnection>) -> Router;
        assert!(true);
    }
}
