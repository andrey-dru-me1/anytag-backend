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

struct UploadedFile {
    original_file_name: Option<String>,
    data: Bytes,
}

/// Image properties obtained from inspecting the uploaded bytes.
#[derive(Debug)]
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
    let image_metadata = inspect_image(&uploaded.data)?;
    let original_file_name = uploaded
        .original_file_name
        .as_deref()
        .unwrap_or(file_sha256_hash);
    check_file_name_length(original_file_name)?;

    let s3_path = &Utc::now().format("images/%Y/%m").to_string();
    let new_image_source = NewImageSource {
        file_size,
        s3_path,
        file_sha256_hash,
        extension: image_metadata.extension,
        mime_type: image_metadata.mime_type,
        bucket_name: &config.s3_media_bucket,
        width: image_metadata.width,
        height: image_metadata.height,
    };
    let new_user_image = NewUserImage {
        original_file_name,
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
        image_metadata.extension,
    );

    Ok(Json(image_dto))
}

/// Parses the numeric image ID from an image name.
///
/// The image name is the `user_images.id`, optionally followed by an
/// extension (e.g. `"1.png"` or `"42"`). The portion before the first dot is
/// parsed as the ID; the extension is ignored.
fn parse_image_id(image_name: &str) -> Result<i32, ApiError> {
    let id_str = match image_name.split('.').next() {
        Some(id) => id,
        None => image_name,
    };

    id_str.parse().map_err(|e| {
        ApiError::builder()
            .code(ApiErrorCode::PathParameterParseError)
            .http_status(StatusCode::BAD_REQUEST)
            .context(format!("'{id_str}' is not a valid image ID: {e}"))
            .message("Valid image id must be provided")
            .build()
    })
}

async fn get_image_by_name(
    image_name: String,
    conn: &mut AsyncPgConnection,
) -> Result<(UserImage, ImageSource), ApiError> {
    let image_id = parse_image_id(&image_name)?;

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

#[cfg(test)]
pub mod tests {
    use super::*;
    use image::ImageFormat;

    // -----------------------------------------------------------------------
    // image fixtures
    // -----------------------------------------------------------------------

    /// Encode a tiny 3x2 RGBA image in the given format.
    pub fn encode_image(format: ImageFormat) -> Vec<u8> {
        let mut img = image::RgbaImage::new(3, 2);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = image::Rgba([x as u8, y as u8, 0, 255]);
        }
        let mut bytes = Cursor::new(Vec::new());
        img.write_to(&mut bytes, format)
            .expect("image encoding should succeed");
        bytes.into_inner()
    }

    pub fn png_bytes() -> Vec<u8> {
        encode_image(ImageFormat::Png)
    }

    pub fn webp_bytes() -> Vec<u8> {
        encode_image(ImageFormat::WebP)
    }

    // -----------------------------------------------------------------------
    // sha256_hex
    // -----------------------------------------------------------------------

    #[test]
    fn test_sha256_hex_empty() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hex_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    // -----------------------------------------------------------------------
    // inspect_image
    // -----------------------------------------------------------------------

    #[test]
    fn test_inspect_image_png() {
        let data = png_bytes();
        let meta = inspect_image(&data).expect("valid png should be inspectable");
        assert_eq!(meta.mime_type, "image/png");
        assert_eq!(meta.extension, "png");
        assert_eq!(meta.width, 3);
        assert_eq!(meta.height, 2);
    }

    #[test]
    fn test_inspect_image_webp() {
        let data = webp_bytes();
        let meta = inspect_image(&data).expect("valid webp should be inspectable");
        assert_eq!(meta.mime_type, "image/webp");
        assert_eq!(meta.extension, "webp");
        assert_eq!(meta.width, 3);
        assert_eq!(meta.height, 2);
    }

    #[test]
    fn test_inspect_image_rejects_non_image_bytes() {
        let err = inspect_image(b"hello, this is not an image").unwrap_err();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert_eq!(*err.error_code(), ApiErrorCode::FileUploadError);
    }

    #[test]
    fn test_inspect_image_rejects_empty_bytes() {
        let err = inspect_image(b"").unwrap_err();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert_eq!(*err.error_code(), ApiErrorCode::FileUploadError);
    }

    #[test]
    fn test_inspect_image_rejects_truncated_png() {
        // Keep only the PNG signature (8 bytes) plus a partial IHDR chunk so the
        // format is still sniffable but the decoder cannot read the dimensions.
        let mut data = png_bytes();
        data.truncate(12);
        let err = inspect_image(&data).unwrap_err();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert_eq!(*err.error_code(), ApiErrorCode::FileUploadError);
    }

    // -----------------------------------------------------------------------
    // check_file_name_length
    // -----------------------------------------------------------------------

    #[test]
    fn test_check_file_name_length_255_ok() {
        let name: String = "a".repeat(255);
        assert!(check_file_name_length(&name).is_ok());
    }

    #[test]
    fn test_check_file_name_length_256_rejected() {
        let name: String = "a".repeat(256);
        let err = check_file_name_length(&name).unwrap_err();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert_eq!(*err.error_code(), ApiErrorCode::FileUploadError);
    }

    #[test]
    fn test_check_file_name_length_empty_ok() {
        assert!(check_file_name_length("").is_ok());
    }

    // -----------------------------------------------------------------------
    // prepare_headers
    // -----------------------------------------------------------------------

    fn sample_image_source() -> ImageSource {
        ImageSource {
            file_sha256_hash: "a".repeat(64),
            s3_path: "images/2026/08".to_string(),
            extension: "png".to_string(),
            file_size: 42,
            mime_type: "image/png".to_string(),
            bucket_name: "test-bucket".to_string(),
            width: 3,
            height: 2,
        }
    }

    #[test]
    fn test_prepare_headers_sets_content_type_and_cache_control() {
        let headers = prepare_headers(&sample_image_source()).expect("headers should build");
        assert_eq!(
            headers
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("image/png")
        );
        assert_eq!(
            headers
                .get(header::CACHE_CONTROL)
                .and_then(|v| v.to_str().ok()),
            Some("public, max-age=2592000")
        );
    }

    #[test]
    fn test_prepare_headers_rejects_unparsable_mime_type() {
        let mut source = sample_image_source();
        source.mime_type = "image/png\nset-cookie: nope".to_string();
        let err = prepare_headers(&source).unwrap_err();
        assert_eq!(err.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(*err.error_code(), ApiErrorCode::S3StorageError);
    }

    // -----------------------------------------------------------------------
    // parse_image_id
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_image_id_plain() {
        assert_eq!(parse_image_id("1").unwrap(), 1);
        assert_eq!(parse_image_id("42").unwrap(), 42);
    }

    #[test]
    fn test_parse_image_id_with_extension() {
        assert_eq!(parse_image_id("1.png").unwrap(), 1);
        assert_eq!(parse_image_id("42.webp").unwrap(), 42);
    }

    #[test]
    fn test_parse_image_id_ignores_extra_dots() {
        assert_eq!(parse_image_id("7.tar.png").unwrap(), 7);
    }

    #[test]
    fn test_parse_image_id_rejects_non_numeric() {
        let err = parse_image_id("abc.png").unwrap_err();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert_eq!(*err.error_code(), ApiErrorCode::PathParameterParseError);
    }

    #[test]
    fn test_parse_image_id_rejects_empty() {
        let err = parse_image_id("").unwrap_err();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
        assert_eq!(*err.error_code(), ApiErrorCode::PathParameterParseError);
    }
}
