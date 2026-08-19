// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use std::io::Cursor;

use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use chrono::Utc;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl, pg::AsyncPgConnection};
use image::ImageReader;
use sha2::{Digest, Sha256};
use tokio_util::io::ReaderStream;

use crate::{
    config::Config,
    dto,
    handlers::{ApiError, ApiErrorCode},
    models::{ImageSource, NewImageSource, NewUserImage, UserImage},
    schema::{
        image_sources,
        user_images::dsl::{id as user_image_id, user_images},
    },
};

/// A `file` field extracted from the multipart request.
struct UploadedFile {
    original_file_name: Option<String>,
    data: Bytes,
}

/// Image properties obtained from inspecting the uploaded bytes.
struct ImageMetadata<'a> {
    mime_type: &'a str,
    extension: &'a str,
    width: i32,
    height: i32,
}

// ---------- multipart parsing ----------

/// Finds the `file` field in the request and reads its raw bytes.
async fn extract_file_field(multipart: &mut Multipart) -> Result<UploadedFile, ApiError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::builder()
            .code(ApiErrorCode::FileUploadError)
            .http_status(e.status())
            .context(format!(
                "failed to extract media file from upload request: {e}"
            ))
            .message("Something is wrong with the uploaded file or file was not provided")
            .build()
    })? {
        if field.name() != Some("file") {
            continue;
        }
        let original_file_name = field.file_name().map(str::to_owned);
        let data = field.bytes().await.map_err(|e| {
            ApiError::builder()
                .code(ApiErrorCode::FileUploadError)
                .http_status(e.status())
                .context(format!("failed to read file contents: {e}"))
                .message("Cannot read the uploaded file contents")
                .build()
        })?;
        return Ok(UploadedFile {
            original_file_name,
            data,
        });
    }

    Err(ApiError::builder()
        .code(ApiErrorCode::FileUploadError)
        .http_status(StatusCode::BAD_REQUEST)
        .context("api request does not have \"file\" field")
        .message("Media file was not provided")
        .build())
}

// ---------- image inspection ----------

/// Validates that `data` decodes as an image and returns its metadata.
fn inspect_image<'a>(data: &[u8]) -> Result<ImageMetadata<'a>, ApiError> {
    let reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| {
            ApiError::builder()
                .code(ApiErrorCode::FileUploadError)
                .http_status(StatusCode::BAD_REQUEST)
                .context(format!("unsupported file format: {e}"))
                .message("Uploaded file format is unsupported")
                .build()
        })?;
    let format = reader.format().ok_or_else(|| {
        ApiError::builder()
            .code(ApiErrorCode::FileUploadError)
            .http_status(StatusCode::BAD_REQUEST)
            .context("unknown file format")
            .message("Uploaded file format is unknown")
            .build()
    })?;
    let extension = format.extensions_str().first().ok_or_else(|| {
        ApiError::builder()
            .code(ApiErrorCode::FileUploadError)
            .http_status(StatusCode::BAD_REQUEST)
            .context("could not recognize extension")
            .message("Uploaded file extension can not be recognized")
            .build()
    })?;
    let (width, height) = reader.into_dimensions().map_err(|e| {
        ApiError::builder()
            .code(ApiErrorCode::FileUploadError)
            .http_status(StatusCode::BAD_REQUEST)
            .context(format!("file corrupted or is not an image: {e}"))
            .message("Uploaded file is corrupted or is not an image")
            .build()
    })?;

    Ok(ImageMetadata {
        mime_type: format.to_mime_type(),
        extension,
        width: width as i32,
        height: height as i32,
    })
}

// ---------- key building ----------

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

// ---------- persistence ----------

async fn insert_image(
    conn: &mut AsyncPgConnection,
    new_image_source: &NewImageSource<'_>,
    new_user_image: &NewUserImage<'_>,
    s3_client: &aws_sdk_s3::Client,
    s3_bucket_name: &str,
    data: Bytes,
) -> Result<UserImage, ApiError> {
    // DB part runs inside a transaction; S3 is deliberately kept outside so a
    // failed image_sources/user_images insert cannot leave an orphaned object.
    let (maybe_image_source, user_image) = conn
        .transaction::<_, ApiError, _>(async |conn| {
            let maybe_image_source = diesel::insert_into(image_sources::dsl::image_sources)
                .values(new_image_source)
                .on_conflict(image_sources::file_sha256_hash)
                .do_nothing()
                .get_result::<ImageSource>(conn)
                .await
                .optional()?;
            let user_image = diesel::insert_into(user_images)
                .values(new_user_image)
                .get_result::<UserImage>(conn)
                .await
                .map_err(|e| {
                    ApiError::builder()
                        .code(ApiErrorCode::DbQueryError)
                        .http_status(StatusCode::INTERNAL_SERVER_ERROR)
                        .context(format!("database image insert query failed: {e}"))
                        .build()
                })?;
            Ok((maybe_image_source, user_image))
        })
        .await?;

    // Only upload the object when a brand-new image_sources row was inserted
    // (on hash conflict the object already exists in the bucket). The key is
    // built from the stored s3_path so it always matches the DB row.
    if let Some(image_source) = maybe_image_source {
        let s3_key = ImageSource::construct_s3_key(
            new_image_source.s3_path,
            &image_source.file_sha256_hash,
            &image_source.extension,
        );
        upload_s3_object(s3_client, s3_bucket_name, &s3_key, data).await?;
    }

    Ok(user_image)
}

// ---------- s3 management ----------

async fn upload_s3_object(
    client: &aws_sdk_s3::Client,
    bucket_name: &str,
    file_name: &str,
    data: Bytes,
) -> Result<(), ApiError> {
    client
        .put_object()
        .bucket(bucket_name)
        .key(file_name)
        .body(data.into())
        .send()
        .await
        .map(|_| ())
        .map_err(|e| {
            ApiError::builder()
                .code(ApiErrorCode::S3StorageError)
                .http_status(StatusCode::INTERNAL_SERVER_ERROR)
                .context(format!(
                    "failed to put object to the s3 bucket '{bucket_name}': {e}"
                ))
                .build()
        })
}

fn check_file_name_length(file_name: &str) -> Result<(), ApiError> {
    let file_name_length = file_name.chars().count();
    if file_name_length > 255 {
        return Err(ApiError::builder()
            .http_status(StatusCode::BAD_REQUEST)
            .code(ApiErrorCode::FileUploadError)
            .context(format!(
                "file name length is {file_name_length}, must be less than 255"
            ))
            .message("File name is too long")
            .build());
    }
    Ok(())
}

// ---------- handler ----------

pub async fn upload_image(
    State(config): State<Config>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let mut conn = config.db_pool.get().await?;

    let uploaded = extract_file_field(&mut multipart).await?;
    let file_size = uploaded.data.len() as i64;
    let file_sha256_hash = &sha256_hex(&uploaded.data);
    let ImageMetadata {
        mime_type,
        extension,
        width,
        height,
    } = inspect_image(&uploaded.data)?;
    let original_file_name = uploaded
        .original_file_name
        .unwrap_or_else(|| file_sha256_hash.clone());
    check_file_name_length(&original_file_name)?;

    let s3_path = &Utc::now().format("images/%Y/%m").to_string();
    let new_image_source = NewImageSource {
        file_size,
        s3_path,
        file_sha256_hash,
        extension,
        mime_type,
        bucket_name: &config.s3_media_bucket,
        width,
        height,
    };
    let new_user_image = NewUserImage {
        original_file_name: &original_file_name,
        file_sha256_hash,
        created_by: 1, // todo: change once jwt is implemented
    };

    let user_image = insert_image(
        &mut conn,
        &new_image_source,
        &new_user_image,
        &config.s3_client,
        &config.s3_media_bucket,
        uploaded.data,
    )
    .await?;
    let image_dto = dto::ImageDto::new(
        new_image_source,
        user_image,
        &format!("{}/api/v1/media/images", config.base_url),
        extension,
    );

    Ok(Json(image_dto))
}

async fn get_image_by_name(
    image_name: String,
    conn: &mut AsyncPgConnection,
) -> Result<(UserImage, ImageSource), ApiError> {
    let id_str = match image_name.split('.').next() {
        Some(id) => id,
        None => &image_name,
    };

    let image_id: i32 = match id_str.parse() {
        Ok(num) => num,
        Err(e) => {
            return Err(ApiError::builder()
                .code(ApiErrorCode::PathParameterParseError)
                .http_status(StatusCode::BAD_REQUEST)
                .context(format!("'{id_str}' is not a valid image ID: {e}"))
                .message("Valid image id must be provided")
                .build());
        }
    };

    user_images
        .inner_join(image_sources::dsl::image_sources)
        .filter(user_image_id.eq(image_id))
        .first::<(UserImage, ImageSource)>(conn)
        .await
        .map_err(|e| {
            ApiError::builder()
                .code(ApiErrorCode::ImageNotFound)
                .http_status(StatusCode::NOT_FOUND)
                .context(format!("image db query failed by id '{image_id}': {e}"))
                .message("Image with such id does not exist")
                .build()
        })
}

fn prepare_headers(image_source: &ImageSource) -> Result<HeaderMap, ApiError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        image_source.mime_type.parse().map_err(|e| {
            ApiError::builder()
                .code(ApiErrorCode::S3StorageError)
                .http_status(StatusCode::INTERNAL_SERVER_ERROR)
                .context(format!(
                    "failed to convert mime type to a header value: {e}"
                ))
                .build()
        })?,
    );
    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=2592000"
            .parse()
            .expect("string literal must be parsed without issues"),
    );

    Ok(headers)
}

pub async fn get_image(
    State(config): State<Config>,
    Path(image_name): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let mut conn = config.db_pool.get().await?;
    let (_user_image, image_source) = get_image_by_name(image_name, &mut conn).await?;

    let s3_object = config
        .s3_client
        .get_object()
        .bucket(&image_source.bucket_name)
        .key(image_source.s3_key())
        .send()
        .await
        .map_err(|e| {
            ApiError::builder()
                .code(ApiErrorCode::S3StorageError)
                .http_status(StatusCode::INTERNAL_SERVER_ERROR)
                .context(format!(
                    "failed to load media file from s3 storage by its hash: '{}': {e}",
                    image_source.file_sha256_hash
                ))
                .build()
        })?;

    let reader = s3_object.body.into_async_read();
    let body_stream = axum::body::Body::from_stream(ReaderStream::new(reader));
    let headers = prepare_headers(&image_source)?;

    Ok((headers, body_stream))
}
