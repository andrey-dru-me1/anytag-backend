// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use anyhow::Context;
use anytag_backend::config::{Config, DbPool};
use anytag_backend::router::create_router;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::{Credentials, Region};
use axum::Router;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use diesel_async::pg::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::deadpool::Pool;

/// Create a test database pool from the `DATABASE_URL` environment variable.
///
/// Uses `max_size=1` to minimise resource usage in tests.
fn test_db_pool() -> anyhow::Result<DbPool> {
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL must be set for integration tests")?;

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder(config)
        .max_size(1)
        .build()
        .context("Failed to create test database pool")
}

/// Transaction guard for test isolation.
///
/// On creation, begins a transaction on the pool's single connection.
/// When dropped, rolls back the transaction, discarding all changes made
/// during the test.
///
/// Tests obtain a pool handle via [`TestTransaction::pool`] and pass it to
/// [`create_router`]. Because the pool is configured with `max_size=1`, every
/// call to `pool.get()` returns the same underlying connection — the one
/// inside this transaction — so all handler operations remain within the
/// transaction boundary. No test data is ever persisted to the database.
pub struct TestTransaction {
    pool: DbPool,
}

impl TestTransaction {
    /// Create a new test transaction.
    pub async fn new() -> anyhow::Result<Self> {
        let pool = test_db_pool()?;
        let mut conn = pool
            .get()
            .await
            .context("Failed to get connection for test transaction")?;
        sql_query("BEGIN")
            .execute(&mut conn)
            .await
            .context("Failed to begin test transaction")?;
        // Drop the connection — it returns to the pool with the transaction active.
        drop(conn);
        Ok(Self { pool })
    }

    /// Return a clone of the pool for use in test handlers.
    pub fn pool(&self) -> DbPool {
        self.pool.clone()
    }
}

/// Build an S3 client pointing at the local docker-compose SeaweedFS endpoint.
///
/// Reads `S3_BASE_URL`, `S3_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` from the
/// environment when set, otherwise falls back to the docker-compose defaults.
/// Ensures the configured bucket exists before returning.
///
/// `#[allow(dead_code)]` because other integration-test binaries that include
/// this module (posts/tags/users) never use the S3 helpers.
#[allow(dead_code)]
async fn s3_client() -> anyhow::Result<(aws_sdk_s3::Client, String)> {
    let base_url = std::env::var("S3_BASE_URL").context("S3_BASE_URL must be set")?;
    let access_key_id = std::env::var("AWS_ACCESS_KEY_ID").context("S3_ACCESS_KEY must be set")?;
    let secret_access_key =
        std::env::var("AWS_SECRET_ACCESS_KEY").context("S3_SECRET_KEY must be set")?;
    let bucket = std::env::var("S3_BUCKET").context("S3_BUCKET must be set")?;

    let credentials = Credentials::new(&access_key_id, &secret_access_key, None, None, "manual");
    let region_provider = RegionProviderChain::default_provider().or_else(Region::new("us-east-1"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;

    let config = aws_sdk_s3::config::Builder::from(&shared_config)
        .credentials_provider(credentials)
        .endpoint_url(&base_url)
        .force_path_style(true)
        .build();
    let client = aws_sdk_s3::Client::from_conf(config);

    // Ensure the bucket exists (idempotent).
    if client.head_bucket().bucket(&bucket).send().await.is_err() {
        client
            .create_bucket()
            .bucket(&bucket)
            .send()
            .await
            .context("Failed to create test S3 bucket")?;
    }

    Ok((client, bucket))
}

/// Build a `Config` backed by a real local SeaweedFS S3 endpoint and an
/// isolated database transaction.
///
/// Intended for image-handler integration tests that must exercise the real
/// `put_object`/`get_object` paths against the docker-compose SeaweedFS
/// container ("without storing images in real storage").
#[allow(dead_code)]
pub async fn test_config_with_s3() -> anyhow::Result<(Config, TestTransaction)> {
    dotenvy::dotenv()?;
    let tx = TestTransaction::new().await?;
    let pool = tx.pool();
    let (client, bucket) = s3_client().await?;
    let base_url = std::env::var("BASE_URL").context("BASE_URL must be set")?;
    let config = Config::new(pool, client, bucket, base_url);
    Ok((config, tx))
}

/// Convenience wrapper for integration tests that bundles a [`TestTransaction`]
/// and a pre-built router.
///
/// # Example
///
/// ```ignore
/// use common::TestApp;
///
/// #[tokio::test]
/// async fn test_something() -> anyhow::Result<()> {
///     let test_app = TestApp::new().await?;
///     let app = test_app.router();
///     // … use `app` as an axum::Router …
///     Ok(())
/// }
/// ```
#[allow(dead_code)]
pub struct TestApp {
    #[allow(dead_code)]
    tx: TestTransaction,
    app: Router,
}

#[allow(dead_code)]
impl TestApp {
    /// Create a new `TestApp` with an isolated database transaction and a router.
    pub async fn new() -> anyhow::Result<Self> {
        let tx = TestTransaction::new().await?;
        let config = Config::from_db_pool(tx.pool());
        let app = create_router(config);
        Ok(Self { tx, app })
    }

    /// Return a clone of the inner router.
    ///
    /// Cloning an axum [`Router`] is cheap (it shares internal state via `Arc`).
    pub fn router(&self) -> Router {
        self.app.clone()
    }
}

// No custom `Drop` needed — `Pool` closes its connections on drop, and
// PostgreSQL automatically rolls back any uncommitted transaction when the
// underlying TCP connection is closed.  When the `TestTransaction` (and all
// cloned pool handles) go out of scope, the pool is dropped, connections are
// closed, and the transaction is rolled back automatically.
