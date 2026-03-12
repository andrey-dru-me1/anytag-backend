// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use chrono::NaiveDateTime;
use diesel::prelude::*;

use super::{TagId, UserId};

#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = crate::schema::tags)]
pub struct Tag {
    pub id: TagId,
    pub user_id: UserId,
    pub label: String,
    pub public: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::tags)]
pub struct NewTag {
    pub user_id: UserId,
    pub label: String,
    pub public: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    #[allow(deprecated)]
    fn test_tag_struct() {
        let tag = Tag {
            id: 1,
            user_id: 42,
            label: "rust".to_string(),
            public: true,
            created_at: NaiveDateTime::from_timestamp_opt(2000, 0).unwrap(),
        };

        assert_eq!(tag.id, 1);
        assert_eq!(tag.user_id, 42);
        assert_eq!(tag.label, "rust");
        assert_eq!(tag.public, true);
        #[allow(deprecated)]
        let timestamp = tag.created_at.timestamp();
        assert_eq!(timestamp, 2000);
    }
}
