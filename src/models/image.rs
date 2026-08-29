// SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
// SPDX-License-Identifier: AGPL-3.0-only

use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable};
use serde::Serialize;

use crate::models::{UserId, UserImageId};

pub type ImageSourceId = i32;

#[derive(Queryable, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::image_sources)]
pub struct ImageSource {
    pub id: ImageSourceId,
    pub s3_key: String,
    pub file_size: i64,
    pub mime_type: String,
    pub bucket_name: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::image_sources)]
pub struct NewImageSource<'a> {
    pub s3_key: &'a str,
    pub file_size: i64,
    pub mime_type: &'a str,
    pub bucket_name: &'a str,
    pub width: i32,
    pub height: i32,
}

#[derive(Queryable, Identifiable, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::user_images)]
pub struct UserImage {
    pub id: UserImageId,
    pub image_source_id: ImageSourceId,
    pub original_file_name: String,
    pub created_by: UserId,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::user_images)]
pub struct NewUserImage<'a> {
    pub image_source_id: ImageSourceId,
    pub original_file_name: &'a str,
    pub created_by: UserId,
}
