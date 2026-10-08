// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

mod common;

use anyhow::Context;
use anytag_backend::models::NewTag;
use anytag_backend::schema::tags::dsl;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use diesel_async::RunQueryDsl;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::common::TestApp;

fn json_get(uri: &str) -> anyhow::Result<axum::http::Request<Body>> {
    axum::http::Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .context("Failed to build GET request")
}

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

/// Helper to insert a test user and return its id.
async fn insert_user(conn: &mut diesel_async::AsyncPgConnection) -> anyhow::Result<i32> {
    use anytag_backend::schema::users::dsl as users_dsl;

    let user = anytag_backend::models::NewUser {
        name: "Tag Test User".to_string(),
        email: "tags_test_user@example.com".to_string(),
        password_hash: "not-a-real-hash".to_string(),
    };

    diesel::insert_into(users_dsl::users)
        .values(&user)
        .returning(users_dsl::id)
        .get_result::<i32>(conn)
        .await
        .context("Failed to insert test user")
}

// ---------------------------------------------------------------------------
// list_tags
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_tags_returns_ok() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;

    let response = test_app.router().oneshot(json_get("/api/v1/tags")?).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let json_body = response_json(response).await?;
    assert!(json_body["tags"].is_array(), "tags should be an array");
    Ok(())
}

#[tokio::test]
async fn test_list_tags_includes_inserted_data() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;

    // Insert a user and some tags within the transaction.
    {
        let mut conn = test_app
            .db_pool
            .get()
            .await
            .context("Failed to get connection")?;
        let user_id = insert_user(&mut conn).await?;

        let new_tags = vec![
            NewTag {
                user_id,
                label: "integration-test-tag-a".to_string(),
                public: true,
            },
            NewTag {
                user_id,
                label: "integration-test-tag-b".to_string(),
                public: false,
            },
        ];

        diesel::insert_into(dsl::tags)
            .values(&new_tags)
            .execute(&mut conn)
            .await
            .context("Failed to insert test tags")?;
    }

    let response = test_app.router().oneshot(json_get("/api/v1/tags")?).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let json_body = response_json(response).await?;
    let tags = json_body["tags"]
        .as_array()
        .context("tags should be an array")?;

    // Our test tags should be present in the response.
    let labels: Vec<&str> = tags.iter().filter_map(|t| t["label"].as_str()).collect();
    assert!(
        labels.contains(&"integration-test-tag-a"),
        "response should contain our test tag A, got: {labels:?}"
    );
    assert!(
        labels.contains(&"integration-test-tag-b"),
        "response should contain our test tag B, got: {labels:?}"
    );

    // Each tag has the expected fields
    for tag in tags {
        assert!(tag["id"].is_number(), "tag should have an id");
        assert!(tag["user_id"].is_number(), "tag should have a user_id");
        assert!(tag["label"].is_string(), "tag should have a label");
        assert!(tag["public"].is_boolean(), "tag should have a public field");
        assert!(tag["created_at"].is_string(), "tag should have created_at");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// list_owned_tags
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_owned_tags_returns_only_current_user_tags() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;

    let email = "owned_tags_user@example.com";
    let password = "CorrectHorseBatteryStaple99!";

    // Create the current user through the API.
    let create_body = json!({
        "name": "Owned Tags User",
        "email": email,
        "password": password,
    });

    let response = test_app
        .router()
        .oneshot(json_post("/api/v1/users", create_body)?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    // Login to get a real access token.
    let login_body = json!({
        "email": email,
        "password": password,
    });

    let response = test_app
        .router()
        .oneshot(json_post("/api/v1/auth/login", login_body)?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let login_json = response_json(response).await?;

    let current_user_id = login_json["user_id"]
        .as_i64()
        .context("user_id should be an integer")? as i32;

    let access_token = login_json["access_token"]
        .as_str()
        .context("access_token should be a string")?
        .to_owned();

    // Insert another user and tags for both users.
    {
        let mut conn = test_app
            .db_pool
            .get()
            .await
            .context("Failed to get connection")?;

        let other_user_id = insert_user(&mut conn).await?;

        let new_tags = vec![
            NewTag {
                user_id: current_user_id,
                label: "current-user-tag".to_string(),
                public: true,
            },
            NewTag {
                user_id: other_user_id,
                label: "other-user-tag".to_string(),
                public: false,
            },
        ];

        diesel::insert_into(dsl::tags)
            .values(&new_tags)
            .execute(&mut conn)
            .await
            .context("Failed to insert test tags")?;
    }

    // Request only the current user's tags.
    let response = test_app
        .router()
        .oneshot(bearer_get("/api/v1/users/me/tags", &access_token)?)
        .await?;

    assert_eq!(response.status(), StatusCode::OK);

    let json_body = response_json(response).await?;
    let tags = json_body["tags"]
        .as_array()
        .context("tags should be an array")?;

    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0]["user_id"], current_user_id);
    assert_eq!(tags[0]["label"], "current-user-tag");

    Ok(())
}

#[tokio::test]
async fn test_list_owned_tags_without_token_returns_unauthorized() -> anyhow::Result<()> {
    let test_app = TestApp::new().await?;

    let response = test_app
        .router()
        .oneshot(json_get("/api/v1/users/me/tags")?)
        .await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    Ok(())
}
