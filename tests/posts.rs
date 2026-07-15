// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

mod common;

use anyhow::Context;
use anytag_backend::models::NewPost;
use anytag_backend::router::create_router;
use anytag_backend::schema::posts::dsl;
use axum::body::Body;
use axum::http::StatusCode;
use common::TestTransaction;
use diesel_async::RunQueryDsl;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

fn json_get(uri: &str) -> anyhow::Result<axum::http::Request<Body>> {
    axum::http::Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .context("Failed to build GET request")
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
        name: "Post Test User".to_string(),
        email: "posts_test_user@example.com".to_string(),
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
// list_posts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_posts_returns_ok() -> anyhow::Result<()> {
    let tx = TestTransaction::new().await?;
    let pool = tx.pool();

    let app = create_router(pool);

    let response = app.oneshot(json_get("/api/v1/posts")?).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let json_body = response_json(response).await?;
    assert!(json_body["posts"].is_array(), "posts should be an array");
    Ok(())
}

#[tokio::test]
async fn test_list_posts_includes_inserted_data() -> anyhow::Result<()> {
    let tx = TestTransaction::new().await?;
    let pool = tx.pool();

    // Insert a user and some posts within the transaction.
    {
        let mut conn = pool.get().await.context("Failed to get connection")?;
        let user_id = insert_user(&mut conn).await?;

        let new_posts = vec![
            NewPost {
                user_id,
                text: "Integration test post A".to_string(),
            },
            NewPost {
                user_id,
                text: "Integration test post B".to_string(),
            },
        ];

        diesel::insert_into(dsl::posts)
            .values(&new_posts)
            .execute(&mut conn)
            .await
            .context("Failed to insert test posts")?;
    }

    let app = create_router(pool);

    let response = app.oneshot(json_get("/api/v1/posts")?).await?;
    assert_eq!(response.status(), StatusCode::OK);

    let json_body = response_json(response).await?;
    let posts = json_body["posts"]
        .as_array()
        .context("posts should be an array")?;

    // Our test posts should be present in the response.
    let texts: Vec<&str> = posts.iter().filter_map(|p| p["text"].as_str()).collect();
    assert!(
        texts.contains(&"Integration test post A"),
        "response should contain our test post A, got: {texts:?}"
    );
    assert!(
        texts.contains(&"Integration test post B"),
        "response should contain our test post B, got: {texts:?}"
    );

    // Each post has the expected fields
    for post in posts {
        assert!(post["id"].is_number(), "post should have an id");
        assert!(post["user_id"].is_number(), "post should have a user_id");
        assert!(post["text"].is_string(), "post should have text");
        assert!(
            post["created_at"].is_string(),
            "post should have created_at"
        );
    }

    Ok(())
}
