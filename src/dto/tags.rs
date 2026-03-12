// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use crate::models::{Tag, TagId, UserId};
use serde::Serialize;

/// Response DTO for a single tag
#[derive(Serialize)]
pub struct TagResponse {
    pub id: TagId,
    pub user_id: UserId,
    pub label: String,
    pub public: bool,
    pub created_at: String,
}

/// Response DTO for a list of tags
#[derive(Serialize)]
pub struct TagsResponse {
    pub tags: Vec<TagResponse>,
}

/// Convert database model to response DTO
impl From<Tag> for TagResponse {
    fn from(tag: Tag) -> Self {
        TagResponse {
            id: tag.id,
            user_id: tag.user_id,
            label: tag.label,
            public: tag.public,
            created_at: tag.created_at.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Tag;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_tag_response_from_tag() {
        let tag = Tag {
            id: 1,
            user_id: 42,
            label: "rust".to_string(),
            public: true,
            created_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap().naive_utc(),
        };

        let response: TagResponse = tag.into();

        assert_eq!(response.id, 1);
        assert_eq!(response.user_id, 42);
        assert_eq!(response.label, "rust");
        assert_eq!(response.public, true);
        assert_eq!(response.created_at, "1970-01-01 00:00:00");
    }

    #[test]
    fn test_list_tags_conversion() {
        let sample_tags = vec![
            Tag {
                id: 1,
                user_id: 42,
                label: "rust".to_string(),
                public: true,
                created_at: DateTime::<Utc>::from_timestamp(1000, 0)
                    .unwrap()
                    .naive_utc(),
            },
            Tag {
                id: 2,
                user_id: 43,
                label: "axum".to_string(),
                public: false,
                created_at: DateTime::<Utc>::from_timestamp(2000, 0)
                    .unwrap()
                    .naive_utc(),
            },
        ];

        let tag_responses: Vec<TagResponse> = sample_tags.into_iter().map(Into::into).collect();

        assert_eq!(tag_responses.len(), 2);
        assert_eq!(tag_responses[0].id, 1);
        assert_eq!(tag_responses[0].label, "rust");
        assert_eq!(tag_responses[0].public, true);
        assert_eq!(tag_responses[1].id, 2);
        assert_eq!(tag_responses[1].label, "axum");
        assert_eq!(tag_responses[1].public, false);
    }
}
