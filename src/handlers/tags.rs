// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use diesel::prelude::*;

use crate::db::{DbPool, get_db_conn};
use crate::dto::{TagResponse, TagsResponse};
use crate::models::Tag;

/// Handler for listing all tags
pub async fn list_tags(
    State(pool): State<DbPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use crate::schema::tags::dsl::*;

    let mut conn = get_db_conn(&pool)?;

    let all_tags = tags
        .order(created_at.desc())
        .load::<Tag>(&mut conn)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load tags: {}", e),
            )
        })?;

    let tag_responses: Vec<TagResponse> = all_tags.into_iter().map(Into::into).collect();

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
        assert!(true);
    }
}
