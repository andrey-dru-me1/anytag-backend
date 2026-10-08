// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Json},
};

use super::auth::get_current_user_id;

use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::config::AppState;
use crate::dto::{TagResponse, TagsResponse};
use crate::handlers::{ApiError, ApiErrorCode};
use crate::models::Tag;

/// Handler for listing all tags
pub async fn list_tags(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    use crate::schema::tags::dsl::*;

    let mut conn = state.db_pool.get().await?;

    let all_tags = tags
        .order(created_at.desc())
        .load::<Tag>(&mut conn)
        .await
        .map_err(|e| {
            (
                ApiErrorCode::DbQueryError,
                format!("Failed to load tags: {}", e),
            )
        })?;

    let tag_responses: Vec<TagResponse> = all_tags.into_iter().map(Into::into).collect();

    Ok(Json(TagsResponse {
        tags: tag_responses,
    }))
}

pub async fn list_owned_tags(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    use crate::schema::tags::dsl::*;

    let current_user_id = get_current_user_id(&headers, &state)?;
    let mut conn = state.db_pool.get().await?;

    let owned_tags = tags
        .filter(user_id.eq(current_user_id))
        .order(created_at.desc())
        .load::<Tag>(&mut conn)
        .await
        .map_err(|e| {
            (
                ApiErrorCode::DbQueryError,
                format!("Failed to load owned tags: {e}"),
            )
        })?;

    let tag_responses: Vec<TagResponse> = owned_tags.into_iter().map(Into::into).collect();

    Ok(Json(TagsResponse {
        tags: tag_responses,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_tags_handler_exists() {
        // Just verify the function exists and can be referenced
        let _ = list_tags;
    }
}
