// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use chrono::NaiveDateTime;
use diesel::prelude::*;

use super::{PostId, UserId};

#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = crate::schema::posts)]
pub struct Post {
    pub id: PostId,
    pub user_id: UserId,
    pub text: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::posts)]
pub struct NewPost {
    pub user_id: UserId,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    #[allow(deprecated)]
    fn test_post_struct() {
        let post = Post {
            id: 1,
            user_id: 42,
            text: "Test post content".to_string(),
            created_at: NaiveDateTime::from_timestamp_opt(1000, 0).unwrap(),
        };

        assert_eq!(post.id, 1);
        assert_eq!(post.user_id, 42);
        assert_eq!(post.text, "Test post content");
        #[allow(deprecated)]
        let timestamp = post.created_at.timestamp();
        assert_eq!(timestamp, 1000);
    }
}
