// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use chrono::NaiveDateTime;
use diesel::prelude::*;

use super::UserId;

#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    #[allow(deprecated)]
    fn test_user_struct() {
        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            created_at: NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
        };

        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Test User");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hash");
        #[allow(deprecated)]
        let timestamp = user.created_at.timestamp();
        assert_eq!(timestamp, 0);

        // Test Debug and Clone
        let cloned = user.clone();
        assert_eq!(cloned.id, user.id);
        assert!(format!("{:?}", user).contains("User"));
    }

    #[test]
    fn test_new_user_struct() {
        let new_user = NewUser {
            name: "New User".to_string(),
            email: "new@example.com".to_string(),
            password_hash: "new_hash".to_string(),
        };

        assert_eq!(new_user.name, "New User");
        assert_eq!(new_user.email, "new@example.com");
        assert_eq!(new_user.password_hash, "new_hash");
    }
}
