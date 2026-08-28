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
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl, pg::AsyncPgConnection};
use image::ImageReader;
use sha2::{Digest, Sha256};
use tokio_util::io::ReaderStream;
use tracing::debug;

use aws_sdk_s3::error::ProvideErrorMetadata;

use crate::{
    config::{AppState, DbPool},
    dto,
    handlers::{ApiError, ApiErrorCode},
    models::{self, ImageSource, NewImageSource, NewUserImage, UserImage},
    schema::{image_sources, user_images},
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

/// Maximum allowed width or height (per side) of an uploaded image.
///
/// Kept well below `i32::MAX` (the type of [`ImageMetadata::width`]) so the
/// `u32 -> i32` conversion is always safe.
const MAX_IMAGE_DIMENSION: u32 = 10_000;

/// Maximum allowed total number of pixels (`width * height`) of an uploaded image.
const MAX_IMAGE_PIXELS: u64 = 40_000_000;

/// Validates that `data` decodes as an image and returns its metadata.
fn inspect_image<'a>(data: &[u8]) -> Result<ImageMetadata<'a>, ApiError> {
    let mut reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|e| {
            ApiError::builder()
                .code(ApiErrorCode::FileUploadError)
                .http_status(StatusCode::BAD_REQUEST)
                .context(format!("unsupported file format: {e}"))
                .message("Uploaded file format is unsupported")
                .build()
        })?;

    // Defense in depth: ask decoders to reject oversized dimensions while
    // reading the header. The explicit checks below remain the source of truth
    // because some decoders do not honor every `Limits` field.
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    reader.limits(limits);

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
        if matches!(
            &e,
            image::ImageError::Limits(limit_err)
                if limit_err.kind() == image::error::LimitErrorKind::DimensionError
        ) {
            return ApiError::builder()
                .code(ApiErrorCode::ImageTooLarge)
                .http_status(StatusCode::PAYLOAD_TOO_LARGE)
                .context(format!("image dimensions exceed allowed limits: {e}"))
                .message("Image dimensions are too large")
                .build();
        }
        ApiError::builder()
            .code(ApiErrorCode::FileUploadError)
            .http_status(StatusCode::BAD_REQUEST)
            .context(format!("file corrupted or is not an image: {e}"))
            .message("Uploaded file is corrupted or is not an image")
            .build()
    })?;

    // Explicit guards against integer overflow (`u32 -> i32`) and excessively
    // large images. These do not rely on the decoder honoring `Limits`.
    let (width, height) = check_image_dimensions(width, height)?;

    Ok(ImageMetadata {
        mime_type: format.to_mime_type(),
        extension,
        width,
        height,
    })
}

/// Validates image dimensions against the configured limits and converts them
/// to the `i32` representation stored in [`ImageMetadata`].
///
/// This guards against both integer overflow (`u32 -> i32` wrapping) and
/// excessively large images, independent of decoder-side limit enforcement.
fn check_image_dimensions(width: u32, height: u32) -> Result<(i32, i32), ApiError> {
    if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
        return Err(image_too_large_error(width, height));
    }
    let total_pixels = u64::from(width) * u64::from(height);
    if total_pixels > MAX_IMAGE_PIXELS {
        return Err(image_too_large_error(width, height));
    }

    // Safe: both dimensions are now `<= MAX_IMAGE_DIMENSION <= i32::MAX`.
    let width = i32::try_from(width).expect("width is bounded by MAX_IMAGE_DIMENSION");
    let height = i32::try_from(height).expect("height is bounded by MAX_IMAGE_DIMENSION");

    Ok((width, height))
}

/// Builds the error returned when an image exceeds the configured size limits.
fn image_too_large_error(width: u32, height: u32) -> ApiError {
    ApiError::builder()
        .code(ApiErrorCode::ImageTooLarge)
        .http_status(StatusCode::PAYLOAD_TOO_LARGE)
        .context(format!(
            "image dimensions {width}x{height} exceed allowed limits \
             (max side {MAX_IMAGE_DIMENSION}, max pixels {MAX_IMAGE_PIXELS})"
        ))
        .message("Image is too large")
        .build()
}

// ---------- key building ----------

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

// ---------- persistence ----------

struct PartialNewUserImage<'a> {
    pub original_file_name: &'a str,
    pub created_by: models::UserId,
}

async fn insert_image(
    pool: &DbPool,
    new_image_source: &NewImageSource<'_>,
    partial_new_user_image: &PartialNewUserImage<'_>,
    s3_client: &aws_sdk_s3::Client,
    s3_bucket_name: &str,
    data: Bytes,
) -> Result<UserImage, ApiError> {
    // Upload to S3 before the DB transaction. This can orphan an object if the
    // inserts below fail, but that is harmless: an orphaned object never fails a
    // user request (unlike an orphaned DB row, which would 404/500 on
    // retrieval), and because the key is content-addressed a retrying uploader
    // will reuse it. See `upload_s3_object` for why the upload itself is
    // idempotent.
    upload_s3_object(s3_client, s3_bucket_name, new_image_source.s3_key, data).await?;

    // A single upsert: `ON CONFLICT ... DO UPDATE` inserts the row if absent and
    // returns the existing one otherwise. Two concurrent uploads of the same
    // content are resolved by the `s3_key` UNIQUE index: the loser blocks until
    // the winner commits, then `DO UPDATE` returns the winner's row. No advisory
    // lock or explicit fetch is needed.
    let user_image = pool
        .get()
        .await?
        .transaction::<_, ApiError, _>(async |conn| {
            let image_source = diesel::insert_into(image_sources::table)
                .values(new_image_source)
                .on_conflict(image_sources::s3_key)
                .do_update()
                .set(image_sources::s3_key.eq(diesel::upsert::excluded(image_sources::s3_key)))
                .get_result::<ImageSource>(conn)
                .await
                .map_err(|e| {
                    ApiError::builder()
                        .code(ApiErrorCode::DbQueryError)
                        .http_status(StatusCode::INTERNAL_SERVER_ERROR)
                        .context(format!("database image_source upsert failed: {e}"))
                        .build()
                })?;

            let new_user_image = NewUserImage {
                image_source_id: image_source.id,
                original_file_name: partial_new_user_image.original_file_name,
                created_by: partial_new_user_image.created_by,
            };
            let user_image = diesel::insert_into(user_images::table)
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
            Ok(user_image)
        })
        .await?;

    Ok(user_image)
}

// ---------- s3 management ----------

async fn upload_s3_object(
    client: &aws_sdk_s3::Client,
    bucket_name: &str,
    file_name: &str,
    data: Bytes,
) -> Result<(), ApiError> {
    let result = client
        .put_object()
        .bucket(bucket_name)
        .key(file_name)
        // Create-only: an existing key returns 412 instead of overwriting.
        // Because the key is content-addressed, an existing object is
        // byte-identical, so 412 is treated as success below.
        .if_none_match("*")
        .body(data.into())
        .send()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(e)
            if e.as_service_error()
                .map(|se| se.code() == Some("PreconditionFailed"))
                .unwrap_or(false) =>
        {
            // Existing object: identical content, nothing to write.
            Ok(())
        }
        Err(e) => Err(ApiError::builder()
            .code(ApiErrorCode::S3StorageError)
            .http_status(StatusCode::INTERNAL_SERVER_ERROR)
            .context(format!(
                "failed to put object to the s3 bucket '{bucket_name}': {e}"
            ))
            .build()),
    }
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
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    let uploaded = extract_file_field(&mut multipart).await?;
    let file_size = uploaded.data.len() as i64;
    let file_sha256_hash = &sha256_hex(&uploaded.data);
    let image_metadata = inspect_image(&uploaded.data)?;
    let original_file_name = uploaded
        .original_file_name
        .as_deref()
        .unwrap_or(file_sha256_hash);
    check_file_name_length(original_file_name)?;

    let new_image_source = NewImageSource {
        s3_key: &format!("images/{}.{}", file_sha256_hash, image_metadata.extension),
        file_size,
        mime_type: image_metadata.mime_type,
        bucket_name: &state.config.s3.media_bucket_name,
        width: image_metadata.width,
        height: image_metadata.height,
    };
    let partial_new_user_image = PartialNewUserImage {
        original_file_name,
        created_by: 1, // todo: change once jwt is implemented
    };

    debug!("Inserting image in db and store to s3");
    let user_image = insert_image(
        &state.db_pool,
        &new_image_source,
        &partial_new_user_image,
        &state.s3_client,
        &state.config.s3.media_bucket_name,
        uploaded.data,
    )
    .await?;
    let image_dto = dto::ImageDto::new(
        new_image_source,
        user_image,
        &format!("{}/api/v1/media/images", state.config.base_url),
        image_metadata.extension,
    );
    debug!("Image is inserted");

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

    user_images::table
        .inner_join(image_sources::table)
        .filter(user_images::id.eq(image_id))
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
    State(state): State<AppState>,
    Path(image_name): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let mut conn = state.db_pool.get().await?;
    let (_user_image, image_source) = get_image_by_name(image_name, &mut conn).await?;

    let s3_object = state
        .s3_client
        .get_object()
        .bucket(&image_source.bucket_name)
        .key(&image_source.s3_key)
        .send()
        .await
        .map_err(|e| {
            ApiError::builder()
                .code(ApiErrorCode::S3StorageError)
                .http_status(StatusCode::INTERNAL_SERVER_ERROR)
                .context(format!(
                    "failed to load media file from s3 storage by its key: '{}': {e}",
                    image_source.s3_key
                ))
                .build()
        })?;

    let reader = s3_object.body.into_async_read();
    let body_stream = axum::body::Body::from_stream(ReaderStream::new(reader));
    let headers = prepare_headers(&image_source)?;

    Ok((headers, body_stream))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageFormat;

    // -----------------------------------------------------------------------
    // image fixtures
    // -----------------------------------------------------------------------

    /// Encode a tiny 3x2 RGBA image in the given format.
    fn encode_image(format: ImageFormat) -> Vec<u8> {
        let mut img = image::RgbaImage::new(3, 2);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = image::Rgba([x as u8, y as u8, 0, 255]);
        }
        let mut bytes = Cursor::new(Vec::new());
        img.write_to(&mut bytes, format)
            .expect("image encoding should succeed");
        bytes.into_inner()
    }

    fn png_bytes() -> Vec<u8> {
        encode_image(ImageFormat::Png)
    }

    fn webp_bytes() -> Vec<u8> {
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
    // check_image_dimensions
    // -----------------------------------------------------------------------

    #[test]
    fn test_check_image_dimensions_accepts_reasonable_size() {
        let (width, height) =
            check_image_dimensions(1920, 1080).expect("1920x1080 should be allowed");
        assert_eq!(width, 1920);
        assert_eq!(height, 1080);
    }

    #[test]
    fn test_check_image_dimensions_accepts_maximum_pixels() {
        // 10_000 x 4_000 == 40_000_000 pixels, exactly at the cap.
        let (width, height) =
            check_image_dimensions(10_000, 4_000).expect("at-cap image should be allowed");
        assert_eq!(width, 10_000);
        assert_eq!(height, 4_000);
    }

    #[test]
    fn test_check_image_dimensions_rejects_side_too_large() {
        let err = check_image_dimensions(MAX_IMAGE_DIMENSION + 1, 10).unwrap_err();
        assert_eq!(err.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(*err.error_code(), ApiErrorCode::ImageTooLarge);
    }

    #[test]
    fn test_check_image_dimensions_rejects_too_many_pixels() {
        // 8_000 x 8_000 == 64_000_000 pixels, over the 40_000_000 cap.
        let err = check_image_dimensions(8_000, 8_000).unwrap_err();
        assert_eq!(err.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(*err.error_code(), ApiErrorCode::ImageTooLarge);
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
            id: 84,
            s3_key: "images/2026/08/a.png".to_string(),
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
