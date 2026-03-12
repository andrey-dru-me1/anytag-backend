// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use crate::models::{Post, PostId, UserId};
use serde::Serialize;

/// Response DTO for a single post
#[derive(Serialize)]
pub struct PostResponse {
    pub id: PostId,
    pub user_id: UserId,
    pub text: String,
    pub created_at: String,
}

/// Response DTO for a list of posts
#[derive(Serialize)]
pub struct PostsResponse {
    pub posts: Vec<PostResponse>,
}

/// Convert database model to response DTO
impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        PostResponse {
            id: post.id,
            user_id: post.user_id,
            text: post.text,
            created_at: post.created_at.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Post;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_post_response_from_post() {
        let post = Post {
            id: 1,
            user_id: 42,
            text: "Test post".to_string(),
            created_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap().naive_utc(),
        };

        let response: PostResponse = post.into();

        assert_eq!(response.id, 1);
        assert_eq!(response.user_id, 42);
        assert_eq!(response.text, "Test post");
        assert_eq!(response.created_at, "1970-01-01 00:00:00");
    }

    #[test]
    fn test_list_posts_conversion() {
        let sample_posts = vec![
            Post {
                id: 1,
                user_id: 42,
                text: "First post".to_string(),
                created_at: DateTime::<Utc>::from_timestamp(1000, 0)
                    .unwrap()
                    .naive_utc(),
            },
            Post {
                id: 2,
                user_id: 43,
                text: "Second post".to_string(),
                created_at: DateTime::<Utc>::from_timestamp(2000, 0)
                    .unwrap()
                    .naive_utc(),
            },
        ];

        let post_responses: Vec<PostResponse> = sample_posts.into_iter().map(Into::into).collect();

        assert_eq!(post_responses.len(), 2);
        assert_eq!(post_responses[0].id, 1);
        assert_eq!(post_responses[0].text, "First post");
        assert_eq!(post_responses[1].id, 2);
        assert_eq!(post_responses[1].text, "Second post");
    }
}
