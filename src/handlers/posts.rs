// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::config::AppState;
use crate::dto::{PostResponse, PostsResponse};
use crate::handlers::{ApiError, ApiErrorCode};
use crate::models::Post;

/// Handler for listing all posts
pub async fn list_posts(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    use crate::schema::posts::dsl::*;

    let mut conn = state.db_pool.get().await?;

    let all_posts = posts
        .order(created_at.desc())
        .load::<Post>(&mut conn)
        .await
        .map_err(|e| {
            (
                ApiErrorCode::DbQueryError,
                format!("Failed to load posts: {}", e),
            )
        })?;

    let post_responses: Vec<PostResponse> = all_posts.into_iter().map(Into::into).collect();

    Ok(Json(PostsResponse {
        posts: post_responses,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_posts_handler_exists() {
        // Just verify the function exists and can be referenced
        let _ = list_posts;
    }
}
