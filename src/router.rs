// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use crate::config::Config;
use crate::handlers;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

/// Create and configure the Axum router
pub fn create_router(config: Config) -> Router {
    let max_size = 10 * 1024 * 1024;

    let api_v1 = Router::new()
        .route("/", get(handlers::health_check))
        .route("/health", get(handlers::health_check))
        .route("/posts", get(handlers::list_posts))
        .route("/tags", get(handlers::list_tags))
        .route("/users", post(handlers::create_user))
        .route("/auth/login", post(handlers::login_user))
        .route("/media/images", post(handlers::upload_image))
        .route("/media/images/{image_name}", get(handlers::get_image))
        .layer(DefaultBodyLimit::max(max_size));

    Router::new().nest("/api/v1", api_v1).with_state(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time check that `create_router` accepts a `Config`.
    #[test]
    fn test_create_router_type_check() {
        let _: fn(Config) -> Router = create_router;
    }
}
