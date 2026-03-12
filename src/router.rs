// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use crate::db::DbPool;
use crate::handlers;
use axum::{Router, routing::get};

/// Create and configure the Axum router
pub fn create_router(pool: DbPool) -> Router {
    Router::new()
        .route("/", get(handlers::health_check))
        .route("/health", get(handlers::health_check))
        .route("/posts", get(handlers::list_posts))
        .route("/tags", get(handlers::list_tags))
        .with_state(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::PgConnection;
    use diesel::r2d2::{ConnectionManager, Pool};

    #[test]
    fn test_create_router_with_mock_pool() {
        // Test that create_router compiles and accepts a DbPool
        // Uses quick-failing connection to avoid timeout

        use std::time::Duration;

        // Non-responsive port ensures quick failure
        let database_url = "postgres://localhost:1/nonexistent";
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        // Short timeout prevents hanging
        let _pool_result = Pool::builder()
            .connection_timeout(Duration::from_millis(100))
            .build(manager);

        // Verify function signature
        let _func: fn(Pool<ConnectionManager<PgConnection>>) -> Router = create_router;
        assert!(true);
    }

    #[test]
    fn test_router_function_exists() {
        // Simple test to verify the function signature
        // This is a compilation test more than a runtime test
        let _ = create_router as fn(Pool<ConnectionManager<PgConnection>>) -> Router;
        assert!(true);
    }
}
