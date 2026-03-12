// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use axum::http::StatusCode;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use dotenv::dotenv;
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

/// Establish a connection pool to the PostgreSQL database
pub fn establish_connection_pool() -> DbPool {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file or environment");

    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool")
}

/// Get a database connection from the pool with proper error handling
///
/// Returns `Ok(PooledConnection)` on success, or `Err((StatusCode, String))` on failure.
pub fn get_db_conn(
    pool: &DbPool,
) -> Result<PooledConnection<ConnectionManager<PgConnection>>, (StatusCode, String)> {
    pool.get().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get database connection: {}", e),
        )
    })
}
