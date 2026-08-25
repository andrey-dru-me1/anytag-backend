// SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
// SPDX-License-Identifier: AGPL-3.0-only

use chrono::NaiveDateTime;
use diesel::{Identifiable, Insertable, Queryable};
use serde::Serialize;

use crate::models::{UserId, UserImageId};

#[derive(Queryable, Serialize, Debug, Clone)]
#[diesel(table_name = crate::schema::image_sources)]
pub struct ImageSource {
    pub file_sha256_hash: String,
    pub s3_key_prefix: String,
    pub extension: String,
    pub file_size: i64,
    pub mime_type: String,
    pub bucket_name: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::image_sources)]
pub struct NewImageSource<'a> {
    pub file_sha256_hash: &'a str,
    pub s3_key_prefix: &'a str,
    pub extension: &'a str,
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
    pub file_sha256_hash: String,
    pub original_file_name: String,
    pub created_by: UserId,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = crate::schema::user_images)]
pub struct NewUserImage<'a> {
    pub file_sha256_hash: &'a str,
    pub original_file_name: &'a str,
    pub created_by: UserId,
}

impl ImageSource {
    pub fn construct_s3_key(s3_key_prefix: &str, file_hash: &str, extension: &str) -> String {
        format!("{s3_key_prefix}/{file_hash}.{extension}")
    }

    pub fn s3_key(&self) -> String {
        Self::construct_s3_key(&self.s3_key_prefix, &self.file_sha256_hash, &self.extension)
    }
}
