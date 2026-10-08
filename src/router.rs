// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use crate::config::AppState;
use crate::handlers::*;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method, header},
    routing::{get, post},
};

use tower_http::cors::CorsLayer;

/// Create and configure the Axum router
pub fn create_router(state: AppState) -> Router {
    let max_size = 10 * 1024 * 1024;

    let cors_origin =
        std::env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://127.0.0.1:18081".to_string());

    let cors = CorsLayer::new()
        .allow_origin(
            cors_origin
                .parse::<HeaderValue>()
                .expect("CORS_ORIGIN must be a valid HTTP origin"),
        )
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let api_v1 = Router::new()
        .route("/", get(health::health_check))
        .route("/health", get(health::health_check))
        .route("/posts", get(posts::list_posts))
        .route("/tags", get(tags::list_tags))
        .route("/users", post(users::create_user))
        .route("/auth/login", post(auth::login_user))
        .route("/users/me", get(auth::get_current_user))
        .route("/auth/refresh", post(auth::refresh_token))
        .route("/auth/logout", post(auth::logout_user))
        .route("/media/images", post(images::upload_image))
        .route("/media/images/{image_name}", get(images::get_image))
        .layer(DefaultBodyLimit::max(max_size));

    Router::new()
        .nest("/api/v1", api_v1)
        .layer(cors)
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time check that `create_router` accepts an `AppState`.
    #[test]
    fn test_create_router_type_check() {
        let _: fn(AppState) -> Router = create_router;
    }
}
