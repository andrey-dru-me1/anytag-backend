// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

mod common;

use anyhow::Context;
use anytag_backend::handlers::ApiErrorCode;
use anytag_backend::{config, schema};
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
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
        assert_eq!(response.status(), StatusCode::OK);
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
