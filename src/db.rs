// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use anyhow::Context;
use diesel_async::{
    pg::AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use dotenv::dotenv;
use std::env;

pub type DbPool = Pool<AsyncPgConnection>;

/// Establish a connection pool to the PostgreSQL database
pub fn establish_connection_pool() -> anyhow::Result<DbPool> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL must be set in .env or environment")?;
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder(config)
        .build()
        .context("Failed to create database connection pool")
        .map_err(Into::into)
}
