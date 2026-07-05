// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use chrono::NaiveDateTime;
use diesel::prelude::*;

use super::UserId;

#[derive(Queryable, Identifiable, Associations, Debug, Clone)]
#[diesel(belongs_to(super::User))]
#[diesel(table_name = crate::schema::refresh_tokens)]
pub struct RefreshToken {
    pub id: i32,
    pub user_id: UserId,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::refresh_tokens)]
pub struct NewRefreshToken {
    pub user_id: UserId,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
}
