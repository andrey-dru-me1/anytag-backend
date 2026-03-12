// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use diesel::prelude::*;

use crate::db::{DbPool, get_db_conn};
use crate::dto::{PostResponse, PostsResponse};
use crate::models::Post;

/// Handler for listing all posts
pub async fn list_posts(
    State(pool): State<DbPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use crate::schema::posts::dsl::*;

    let mut conn = get_db_conn(&pool)?;

    let all_posts = posts
        .order(created_at.desc())
        .load::<Post>(&mut conn)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
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
        assert!(true);
    }
}
