// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::dto::{TagResponse, TagsResponse};
use crate::models::Tag;
use crate::{
    db::DbPool,
    handlers::{ErrCode, HandlerErr},
};

/// Handler for listing all tags
pub async fn list_tags(State(pool): State<DbPool>) -> Result<impl IntoResponse, HandlerErr> {
    use crate::schema::tags::dsl::*;

    let mut conn = pool.get().await?;

    let all_tags = tags
        .order(created_at.desc())
        .load::<Tag>(&mut conn)
        .await
        .map_err(|e| (ErrCode::DbQueryError, format!("Failed to load tags: {}", e)))?;

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
    }
}
