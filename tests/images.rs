// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

mod common;

use anyhow::Context;
use anytag_backend::config::AppConfig;
use anytag_backend::handlers::ApiErrorCode;
use anytag_backend::{config, schema};
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use http_body_util::BodyExt;
use image::ImageFormat;
use serde_json::Value;
use std::io::Cursor;
use tower::ServiceExt;

use crate::common::TestApp;

// fixme: the following two functions are duplicated in handlers::images

/// Encode a tiny RGBA image in the given format.
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

async fn response_json(response: axum::response::Response) -> anyhow::Result<Value> {
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .context("Failed to collect response body")?
        .to_bytes();
    serde_json::from_slice(&body_bytes).context("Response body is not valid JSON")
}

async fn response_bytes(response: axum::response::Response) -> anyhow::Result<Vec<u8>> {
    let body_bytes = response
        .into_body()
        .collect()
        .await
        .context("Failed to collect response body")?
        .to_bytes();
    Ok(body_bytes.to_vec())
}

/// Upload `data` as a multipart `file` field.
fn multipart_upload(data: &[u8], file_name: &str) -> anyhow::Result<Request<Body>> {
    let boundary = "----anytag-test-boundary";
    let mut body: Vec<u8> = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    Request::builder()
        .method("POST")
        .uri("/api/v1/media/images")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .context("Failed to build multipart upload request")
}

/// Upload `data` with an empty multipart body (no `file` field).
fn multipart_upload_empty() -> anyhow::Result<Request<Body>> {
    let boundary = "----anytag-test-boundary";
    Request::builder()
        .method("POST")
        .uri("/api/v1/media/images")
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(format!("--{boundary}--\r\n")))
        .context("Failed to build empty multipart upload request")
}

/// Insert a user with a fixed id so uploads (which hardcode `created_by: 1`)
/// satisfy the `user_images.created_by` foreign key.
async fn insert_user_with_id(pool: &config::DbPool, id: i32) -> anyhow::Result<()> {
    let mut conn = pool.get().await.context("Failed to get test connection")?;
    // `ON CONFLICT (id) DO NOTHING` keeps this robust against a pre-existing
    // committed row with the same id (e.g. the local dev database where a real
    // `users` row with id=1 already exists); the foreign key is satisfied
    // either way. In CI the database is fresh, so the row is actually created.
    #[derive(diesel::Insertable)]
    #[diesel(table_name = crate::schema::users)]
    struct NewUser<'a> {
        id: i32,
        name: &'a str,
        email: &'a str,
        password_hash: &'a str,
    }
    let new_user = NewUser {
        id,
        name: &format!("User {id}"),
        email: &format!("user{id}@example.com"),
        password_hash: "not-a-real-hash",
    };
    let _ = diesel::insert_into(schema::users::dsl::users)
        .values(&new_user)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await;
    Ok(())
}

async fn run_test_with_s3<Fut>(test_logic: impl FnOnce(TestApp) -> Fut) -> anyhow::Result<()>
where
    Fut: Future<Output = anyhow::Result<()>>,
{
    TestApp::with_temporary_s3_bucket(async |test_app| {
        insert_user_with_id(&test_app.db_pool, 1).await?;
        test_logic(test_app).await
    })
    .await
}

// ---------------------------------------------------------------------------
// upload_image
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_upload_image_success() -> anyhow::Result<()> {
    run_test_with_s3(async |test_app| {
        let response = test_app
            .router()
            .oneshot(multipart_upload(&png_bytes(), "photo.png")?)
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let json = response_json(response).await?;
        assert!(json["id"].is_number(), "id should be a number");
        assert_eq!(json["original_file_name"], "photo.png");
        assert_eq!(json["file_size"], png_bytes().len() as i64);
        assert_eq!(json["width"], 3);
        assert_eq!(json["height"], 2);
        assert_eq!(json["created_by"], 1);
        let url = json["access_url"].as_str().context("access_url missing")?;
        assert!(url.ends_with(".png"), "access_url should end with .png");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_upload_image_without_file_field() -> anyhow::Result<()> {
    run_test_with_s3(async |test_app| {
        let response = test_app.router().oneshot(multipart_upload_empty()?).await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = response_json(response).await?;
        assert_eq!(json["code"], ApiErrorCode::FileUploadError.as_ref());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_upload_image_rejects_non_image_bytes() -> anyhow::Result<()> {
    run_test_with_s3(async |test_app| {
        let response = test_app
            .router()
            .oneshot(multipart_upload(
                b"hello, definitely not an image",
                "fake.png",
            )?)
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = response_json(response).await?;
        assert_eq!(json["code"], ApiErrorCode::FileUploadError.as_ref());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_upload_image_rejects_overlong_file_name() -> anyhow::Result<()> {
    run_test_with_s3(async |test_app| {
        let long_name: String = "a".repeat(256);
        let response = test_app
            .router()
            .oneshot(multipart_upload(&png_bytes(), &long_name)?)
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = response_json(response).await?;
        assert_eq!(json["code"], ApiErrorCode::FileUploadError.as_ref());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_upload_image_rejects_oversized_body() -> anyhow::Result<()> {
    // The body limit is enforced by the router before the handler runs,
    // so a mock S3 client is sufficient and no DB/S3 state is touched.
    let test_app = TestApp::new().await?;

    // 10 MB + 1 byte of payload exceeds the router's DefaultBodyLimit,
    // which axum turns into an empty 413 Payload Too Large response.
    let big_data = vec![0u8; 10 * 1024 * 1024 + 1];
    let response = test_app
        .router()
        .oneshot(multipart_upload(&big_data, "huge.png")?)
        .await?;
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    Ok(())
}

#[tokio::test]
async fn test_upload_image_dedup_same_bytes() -> anyhow::Result<()> {
    run_test_with_s3(async |test_app| {
        let data = png_bytes();
        let resp1 = test_app
            .router()
            .oneshot(multipart_upload(&data, "a.png")?)
            .await?;
        let resp2 = test_app
            .router()
            .oneshot(multipart_upload(&data, "b.png")?)
            .await?;
        assert_eq!(resp1.status(), StatusCode::OK);
        assert_eq!(resp2.status(), StatusCode::OK);

        let json1 = response_json(resp1).await?;
        let json2 = response_json(resp2).await?;
        // Identical bytes share the same image_source but create distinct user_images.
        assert_ne!(json1["id"], json2["id"]);
        assert_eq!(json1["original_file_name"], "a.png");
        assert_eq!(json2["original_file_name"], "b.png");

        // Both should be retrievable.
        let get1 = test_app
            .router()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/media/images/{}.png", json1["id"]))
                    .body(Body::empty())
                    .context("Failed to build GET request")?,
            )
            .await?;
        let get2 = test_app
            .router()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/media/images/{}.png", json2["id"]))
                    .body(Body::empty())
                    .context("Failed to build GET request")?,
            )
            .await?;
        assert_eq!(get1.status(), StatusCode::OK);
        assert_eq!(get2.status(), StatusCode::OK);
        assert_eq!(response_bytes(get1).await?, response_bytes(get2).await?);
        Ok(())
    })
    .await
}

// ---------------------------------------------------------------------------
// get_image
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_image_success() -> anyhow::Result<()> {
    run_test_with_s3(async |test_app| {
        let upload = test_app
            .router()
            .oneshot(multipart_upload(&png_bytes(), "photo.png")?)
            .await?;
        assert_eq!(upload.status(), StatusCode::OK);
        let json = response_json(upload).await?;
        let id = json["id"].as_i64().context("id missing")?;

        let response = test_app
            .router()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/media/images/{id}.png"))
                    .body(Body::empty())
                    .context("Failed to build GET request")?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK, "{response:#?}");
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok()),
            Some("image/png")
        );
        assert_eq!(
            response
                .headers()
                .get(header::CACHE_CONTROL)
                .and_then(|v| v.to_str().ok()),
            Some("public, max-age=2592000")
        );
        assert_eq!(response_bytes(response).await?, png_bytes());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_get_image_not_found() -> anyhow::Result<()> {
    run_test_with_s3(async |test_app| {
        let response = test_app
            .router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/media/images/999999.png")
                    .body(Body::empty())
                    .context("Failed to build GET request")?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let json = response_json(response).await?;
        assert_eq!(json["code"], ApiErrorCode::ImageNotFound.as_ref());
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_get_image_invalid_id() -> anyhow::Result<()> {
    run_test_with_s3(async |test_app| {
        let response = test_app
            .router()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/media/images/not-an-id.png")
                    .body(Body::empty())
                    .context("Failed to build GET request")?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let json = response_json(response).await?;
        assert_eq!(json["code"], ApiErrorCode::PathParameterParseError.as_ref());
        Ok(())
    })
    .await
}

// ---------------------------------------------------------------------------
// concurrency
// ---------------------------------------------------------------------------

/// Two concurrent uploads of the same bytes must both succeed, produce
/// distinct `user_images` rows, and yield exactly one `image_sources` row.
/// Without serialization, one of the uploads would 500 on the `s3_key` unique
/// constraint (and one S3 object would silently overwrite the other).
///
/// fixme: this test is not actually concurrent. The test pool uses `max_size=1`
/// and `begin_test_transaction` (`tests/common/mod.rs`), so the two
/// `tokio::join!` uploads serialize on the pool connection acquired at the top
/// of `upload_image`. The real race (loser blocking on the `s3_key` UNIQUE
/// index until the winner commits) is never exercised here.
#[tokio::test]
async fn test_upload_concurrent_identical_bytes() -> anyhow::Result<()> {
    run_test_with_s3(async |test_app| {
        let data = png_bytes();

        let (a, b) = tokio::join!(
            test_app.router().oneshot(multipart_upload(&data, "a.png")?),
            test_app.router().oneshot(multipart_upload(&data, "b.png")?)
        );
        let (resp_a, resp_b) = (a?, b?);
        assert_eq!(
            resp_a.status(),
            StatusCode::OK,
            "first concurrent upload failed"
        );
        assert_eq!(
            resp_b.status(),
            StatusCode::OK,
            "second concurrent upload failed"
        );

        let json_a = response_json(resp_a).await?;
        let json_b = response_json(resp_b).await?;
        assert_ne!(
            json_a["id"], json_b["id"],
            "each upload must create its own user_images row"
        );

        // Exactly one image_sources row must exist for this content. Count by the
        // content-addressed s3_key (not just mime_type) so a pre-existing committed
        // row from a previous run cannot skew the assertion.
        use sha2::{Digest, Sha256};
        let expected_key = format!("images/{}.png", hex::encode(Sha256::digest(&data)));

        let mut conn = test_app
            .db_pool
            .get()
            .await
            .context("Failed to get test connection")?;
        let count = schema::image_sources::table
            .filter(schema::image_sources::s3_key.eq(&expected_key))
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .context("Failed to count image_sources rows")?;
        assert_eq!(
            count, 1,
            "concurrent identical uploads must deduplicate to one image_sources row"
        );

        Ok(())
    })
    .await
}

// ---------------------------------------------------------------------------
// re-upload after image_sources row is gone (conditional PUT path)
// ---------------------------------------------------------------------------

/// Upload an image, delete the `image_sources` (and `user_images`) rows, then
/// re-upload the same bytes. The S3 object still exists, so the app's
/// conditional PUT must get `412` and proceed to re-insert the DB row instead
/// of failing or overwriting.
#[tokio::test]
async fn test_reupload_when_object_remains_after_db_row_deleted() -> anyhow::Result<()> {
    run_test_with_s3(async |test_app| {
        let data = png_bytes();
        let resp = test_app
            .router()
            .oneshot(multipart_upload(&data, "photo.png")?)
            .await?;
        assert_eq!(resp.status(), StatusCode::OK);
        let json = response_json(resp).await?;
        let first_id = json["id"].as_i64().context("id missing")?;

        // Delete the DB rows but leave the S3 object in place. The connection
        // must be dropped (scoped block) before the re-upload request below:
        // the test pool has max_size=1, so holding it here would deadlock the
        // re-upload's own `db_pool.get()`.
        {
            let mut conn = test_app
                .db_pool
                .get()
                .await
                .context("Failed to get test connection")?;
            diesel::delete(schema::user_images::table)
                .execute(&mut conn)
                .await
                .context("Failed to delete user_images rows")?;
            diesel::delete(schema::image_sources::table)
                .execute(&mut conn)
                .await
                .context("Failed to delete image_sources rows")?;
        }

        // Re-upload the identical bytes: the app must reuse the existing S3
        // object (conditional PUT returns 412, treated as success) and recreate
        // the DB rows.
        let resp2 = test_app
            .router()
            .oneshot(multipart_upload(&data, "photo.png")?)
            .await?;
        assert_eq!(
            resp2.status(),
            StatusCode::OK,
            "re-upload after DB row deletion should succeed"
        );
        let json2 = response_json(resp2).await?;
        assert_ne!(
            json2["id"], first_id,
            "re-upload creates a new user_images row"
        );

        // The re-uploaded image must be retrievable.
        let get = test_app
            .router()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/media/images/{}.png", json2["id"]))
                    .body(Body::empty())
                    .context("Failed to build GET request")?,
            )
            .await?;
        assert_eq!(
            get.status(),
            StatusCode::OK,
            "re-upped image should be retrievable"
        );
        assert_eq!(
            response_bytes(get).await?,
            data,
            "retrieved bytes must match the upload"
        );

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_orphan_s3_object_deleted() -> anyhow::Result<()> {
    // Point the pool at an unreachable database. `upload_image` puts the object
    // to S3 first, then fails to obtain a DB connection; the app must delete the
    // just-uploaded object rather than leaving an orphan behind.
    let mut config = AppConfig::from_dotenv()?;
    config.database_url = "postgres://wrong:user@wrong:host/wrong".to_string();
    TestApp::from_config_with_temporary_s3_bucket(config, async |test_app| -> anyhow::Result<()> {
        let response = test_app
            .router()
            .oneshot(multipart_upload(&png_bytes(), "photo.png")?)
            .await?;
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let json = response_json(response).await?;
        assert_eq!(json["code"], ApiErrorCode::DbConnectionError.as_ref());

        // The upload reached S3, but the DB insert failed, so the app must have
        // deleted the orphaned object: the freshly created bucket must be empty.
        let objects = test_app
            .s3_client
            .list_objects_v2()
            .bucket(&test_app.config.s3.media_bucket_name)
            .send()
            .await?;
        assert!(
            objects.contents().is_empty(),
            "orphaned S3 object must be deleted after a failed DB insert"
        );

        Ok(())
    })
    .await
}
