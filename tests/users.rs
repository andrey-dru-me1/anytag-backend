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

fn bearer_get(uri: &str, token: &str) -> anyhow::Result<Request<Body>> {
    Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .context("Failed to build authenticated GET request")
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

async fn create_test_user(
    test_app: &TestApp,
    name: &str,
    email: &str,
    password: &str,
) -> anyhow::Result<()> {
    let body = json!({
        "name": name,
        "email": email,
        "password": password,
    });

    let response = test_app
        .router()
        .oneshot(json_post("/api/v1/users", body)?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

async fn login_test_user(test_app: &TestApp, email: &str, password: &str) -> anyhow::Result<Value> {
    let body = json!({
        "email": email,
        "password": password,
    });

    let response = test_app
        .router()
        .oneshot(json_post("/api/v1/auth/login", body)?)
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    response_json(response).await
}

fn token_from_response<'a>(response: &'a Value, field: &str) -> anyhow::Result<&'a str> {
    response[field]
        .as_str()
        .with_context(|| format!("Response field '{field}' is not a string"))
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
    assert!(!token_from_response(&json_body, "access_token")?.is_empty());
    assert!(!token_from_response(&json_body, "refresh_token")?.is_empty());
    assert_eq!(json_body["token_type"], "Bearer");
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

// ---------------------------------------------------------------------------
// get_current_user
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_current_user_without_token() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/users/me")
        .body(Body::empty())?;
    let response = test_app.router().oneshot(request).await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json_body = response_json(response).await?;
    assert_eq!(json_body["code"], ApiErrorCode::InvalidToken.as_ref());
    Ok(())
}

#[tokio::test]
async fn test_get_current_user_with_access_token() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;
    let name = "Current User";
    let email = "test_current_user@example.com";
    let password = "CorrectHorseBatteryStaple99!";
    create_test_user(&test_app, name, email, password).await?;
    let login = login_test_user(&test_app, email, password).await?;

    let response = test_app
        .router()
        .oneshot(bearer_get(
            "/api/v1/users/me",
            token_from_response(&login, "access_token")?,
        )?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let json_body = response_json(response).await?;
    assert_eq!(json_body["id"], login["user_id"]);
    assert_eq!(json_body["name"], name);
    assert_eq!(json_body["email"], email);
    Ok(())
}

#[tokio::test]
async fn test_get_current_user_rejects_refresh_token() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;
    let email = "test_current_user_refresh@example.com";
    let password = "CorrectHorseBatteryStaple99!";
    create_test_user(&test_app, "Refresh As Access", email, password).await?;
    let login = login_test_user(&test_app, email, password).await?;

    let response = test_app
        .router()
        .oneshot(bearer_get(
            "/api/v1/users/me",
            token_from_response(&login, "refresh_token")?,
        )?)
        .await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json_body = response_json(response).await?;
    assert_eq!(json_body["code"], ApiErrorCode::InvalidToken.as_ref());
    Ok(())
}

// ---------------------------------------------------------------------------
// refresh_token
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_refresh_token_rotates_token_pair() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;
    let email = "test_refresh_rotation@example.com";
    let password = "CorrectHorseBatteryStaple99!";
    create_test_user(&test_app, "Refresh Rotation", email, password).await?;
    let login = login_test_user(&test_app, email, password).await?;
    let old_refresh_token = token_from_response(&login, "refresh_token")?;

    let response = test_app
        .router()
        .oneshot(json_post(
            "/api/v1/auth/refresh",
            json!({ "refresh_token": old_refresh_token }),
        )?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);
    let json_body = response_json(response).await?;
    assert!(!token_from_response(&json_body, "access_token")?.is_empty());
    assert_ne!(
        token_from_response(&json_body, "refresh_token")?,
        old_refresh_token
    );
    assert_eq!(json_body["token_type"], "Bearer");
    Ok(())
}

#[tokio::test]
async fn test_refresh_token_rejects_rotated_token() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;
    let email = "test_refresh_reuse@example.com";
    let password = "CorrectHorseBatteryStaple99!";
    create_test_user(&test_app, "Refresh Reuse", email, password).await?;
    let login = login_test_user(&test_app, email, password).await?;
    let old_refresh_token = token_from_response(&login, "refresh_token")?;

    let first_response = test_app
        .router()
        .oneshot(json_post(
            "/api/v1/auth/refresh",
            json!({ "refresh_token": old_refresh_token }),
        )?)
        .await?;
    assert_eq!(first_response.status(), StatusCode::OK);

    let second_response = test_app
        .router()
        .oneshot(json_post(
            "/api/v1/auth/refresh",
            json!({ "refresh_token": old_refresh_token }),
        )?)
        .await?;

    assert_eq!(second_response.status(), StatusCode::UNAUTHORIZED);
    let json_body = response_json(second_response).await?;
    assert_eq!(json_body["code"], ApiErrorCode::InvalidToken.as_ref());
    Ok(())
}

#[tokio::test]
async fn test_refresh_token_rejects_access_token() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;
    let email = "test_refresh_access@example.com";
    let password = "CorrectHorseBatteryStaple99!";
    create_test_user(&test_app, "Access As Refresh", email, password).await?;
    let login = login_test_user(&test_app, email, password).await?;

    let response = test_app
        .router()
        .oneshot(json_post(
            "/api/v1/auth/refresh",
            json!({
                "refresh_token": token_from_response(&login, "access_token")?
            }),
        )?)
        .await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json_body = response_json(response).await?;
    assert_eq!(json_body["code"], ApiErrorCode::InvalidToken.as_ref());
    Ok(())
}

// ---------------------------------------------------------------------------
// logout_user
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_logout_user_revokes_refresh_token() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;
    let email = "test_logout@example.com";
    let password = "CorrectHorseBatteryStaple99!";
    create_test_user(&test_app, "Logout User", email, password).await?;
    let login = login_test_user(&test_app, email, password).await?;
    let refresh_token = token_from_response(&login, "refresh_token")?;

    let logout_response = test_app
        .router()
        .oneshot(json_post(
            "/api/v1/auth/logout",
            json!({ "refresh_token": refresh_token }),
        )?)
        .await?;
    assert_eq!(logout_response.status(), StatusCode::OK);
    let logout_json = response_json(logout_response).await?;
    assert_eq!(logout_json["message"], "logout successful");

    let refresh_response = test_app
        .router()
        .oneshot(json_post(
            "/api/v1/auth/refresh",
            json!({ "refresh_token": refresh_token }),
        )?)
        .await?;
    assert_eq!(refresh_response.status(), StatusCode::UNAUTHORIZED);
    let refresh_json = response_json(refresh_response).await?;
    assert_eq!(refresh_json["code"], ApiErrorCode::InvalidToken.as_ref());
    Ok(())
}
