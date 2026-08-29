// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

mod common;

use anyhow::Context;
use anytag_backend::handlers::ApiErrorCode;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::TestApp;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

fn json_post(uri: &str, body: Value) -> anyhow::Result<Request<Body>> {
    let json_body = serde_json::to_string(&body).context("Failed to serialize request body")?;
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(json_body))
        .context("Failed to build POST request")
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

// ---------------------------------------------------------------------------
// create_user
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_user_success() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;
    let email = "test_create_success@example.com";

    let body = json!({
        "name": "Test User",
        "email": email,
        "password": "CorrectHorseBatteryStaple99!"
    });

    let response = test_app
        .router()
        .oneshot(json_post("/api/v1/users", body)?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let json_body = response_json(response).await?;
    assert_eq!(json_body["message"], "user created");
    assert_eq!(json_body["name"], "Test User");
    assert_eq!(json_body["email"], email);
    Ok(())
}

#[tokio::test]
async fn test_create_user_invalid_email() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;

    let body = json!({
        "name": "Bad Email",
        "email": "not-an-email",
        "password": "CorrectHorseBatteryStaple99!"
    });

    let response = test_app
        .router()
        .oneshot(json_post("/api/v1/users", body)?)
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let json_body = response_json(response).await?;
    assert_eq!(json_body["code"], ApiErrorCode::InvalidEmail.as_ref());
    Ok(())
}

#[tokio::test]
async fn test_create_user_weak_password() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;

    let body = json!({
        "name": "Weak PW",
        "email": "weak@example.com",
        "password": "12345678"
    });

    let response = test_app
        .router()
        .oneshot(json_post("/api/v1/users", body)?)
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let json_body = response_json(response).await?;
    assert_eq!(json_body["code"], ApiErrorCode::WeakPassword.as_ref());
    Ok(())
}

#[tokio::test]
async fn test_create_user_duplicate_email() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;
    let email = "test_duplicate@example.com";

    let body = json!({
        "name": "Original",
        "email": email,
        "password": "CorrectHorseBatteryStaple99!"
    });

    // First creation — should succeed
    let response1 = test_app
        .router()
        .oneshot(json_post("/api/v1/users", body.clone())?)
        .await?;
    assert_eq!(response1.status(), StatusCode::OK);

    // Second creation with the same email — should fail
    let response2 = test_app
        .router()
        .oneshot(json_post("/api/v1/users", body)?)
        .await?;
    // DB_QUERY_ERROR via From<(ApiErrorCode, String)> defaults to 500
    assert_eq!(response2.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let json_body = response_json(response2).await?;
    assert_eq!(json_body["code"], ApiErrorCode::DbQueryError.as_ref());
    Ok(())
}

// ---------------------------------------------------------------------------
// login_user
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_login_user_success() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;
    let email = "test_login_success@example.com";
    let password = "CorrectHorseBatteryStaple99!";

    // Create user first
    let create_body = json!({
        "name": "Login Test",
        "email": email,
        "password": password,
    });

    let create_resp = test_app
        .router()
        .oneshot(json_post("/api/v1/users", create_body)?)
        .await?;
    assert_eq!(create_resp.status(), StatusCode::OK);

    // Login with correct credentials
    let login_body = json!({
        "email": email,
        "password": password,
    });

    let response = test_app
        .router()
        .oneshot(json_post("/api/v1/auth/login", login_body)?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let json_body = response_json(response).await?;
    assert_eq!(json_body["message"], "login successful");
    assert_eq!(json_body["email"], email);
    assert!(json_body["user_id"].is_number());
    Ok(())
}

#[tokio::test]
async fn test_login_user_wrong_password() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;
    let email = "test_login_wrong_pw@example.com";

    // Create user
    let create_body = json!({
        "name": "Wrong PW",
        "email": email,
        "password": "CorrectHorseBatteryStaple99!",
    });

    let _ = test_app
        .router()
        .oneshot(json_post("/api/v1/users", create_body)?)
        .await?;

    // Login with wrong password
    let login_body = json!({
        "email": email,
        "password": "WrongPassword123!",
    });

    let response = test_app
        .router()
        .oneshot(json_post("/api/v1/auth/login", login_body)?)
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = response_json(response).await?;
    assert_eq!(json_body["code"], ApiErrorCode::InvalidCredentials.as_ref());
    Ok(())
}

#[tokio::test]
async fn test_login_user_not_found() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;

    let login_body = json!({
        "email": "nonexistent@example.com",
        "password": "SomePassword123!",
    });

    let response = test_app
        .router()
        .oneshot(json_post("/api/v1/auth/login", login_body)?)
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = response_json(response).await?;
    // Same error code as wrong password (security best practice)
    assert_eq!(json_body["code"], ApiErrorCode::InvalidCredentials.as_ref());
    Ok(())
}
