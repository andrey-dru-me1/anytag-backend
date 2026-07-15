// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use anyhow::Context;
use anytag_backend::db::DbPool;
use anytag_backend::router::create_router;
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
    dotenv::dotenv().ok();
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
        let pool = tx.pool();
        let app = create_router(pool);
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
