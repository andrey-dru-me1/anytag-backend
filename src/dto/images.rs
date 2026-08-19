// SPDX-FileCopyrightText: 2026 The Anytag Backend Authors
// SPDX-License-Identifier: AGPL-3.0-only

use chrono::NaiveDateTime;
use serde::Serialize;

use crate::models::{NewImageSource, UserId, UserImage, UserImageId};

#[derive(Serialize)]
pub struct ImageDto {
    pub id: UserImageId,
    pub original_file_name: String,
    pub access_url: String,
    pub file_size: i64,
    pub width: i32,
    pub height: i32,
    pub created_by: UserId,
    pub created_at: NaiveDateTime,
}

impl ImageDto {
    pub fn new(
        image_source: NewImageSource,
        user_image: UserImage,
        api_route: &str,
        extension: &str,
    ) -> Self {
        ImageDto {
            id: user_image.id,
            original_file_name: user_image.original_file_name,
            access_url: format!("{api_route}/{}.{extension}", user_image.id),
            file_size: image_source.file_size,
            width: image_source.width,
            height: image_source.height,
            created_by: user_image.created_by,
            created_at: user_image.created_at,
        }
    }
}
