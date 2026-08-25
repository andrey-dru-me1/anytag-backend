// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use anyhow::Context;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{
    config::{Credentials, Region},
    error::SdkError,
    operation::head_bucket::HeadBucketError,
};
use diesel_async::{
    pg::AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use dotenvy::dotenv;
use std::env;

pub type DbPool = Pool<AsyncPgConnection>;

#[derive(Clone)]
pub struct Config {
    pub db_pool: DbPool,
    pub s3_client: aws_sdk_s3::Client,
    pub s3_media_bucket: String,
    pub base_url: String,
}

impl Config {
    pub async fn from_env() -> anyhow::Result<Config> {
        dotenv().ok();

        let (s3_client, s3_media_bucket) = Self::setup_s3_client().await?;
        let base_url = load_env("BASE_URL")?;
        Ok(Config {
            db_pool: Self::setup_database()?,
            s3_client,
            s3_media_bucket,
            base_url,
        })
    }

    fn setup_database() -> anyhow::Result<DbPool> {
        let db_url = load_env("DATABASE_URL")?;
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
        Pool::builder(config)
            .build()
            .context("Failed to create database connection pool")
    }

    async fn setup_s3_client() -> anyhow::Result<(aws_sdk_s3::Client, String)> {
        let access_key_id = load_env("AWS_ACCESS_KEY_ID")?;
        let secret_access_key = load_env("AWS_SECRET_ACCESS_KEY")?;
        let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "manual");

        let region_provider =
            RegionProviderChain::default_provider().or_else(Region::new("us-east-1"));
        let shared_config = aws_config::from_env().region(region_provider).load().await;

        let base_url = load_env("S3_BASE_URL")?;
        let config = aws_sdk_s3::config::Builder::from(&shared_config)
            .credentials_provider(credentials)
            .endpoint_url(base_url)
            .force_path_style(true)
            .build();
        let client = aws_sdk_s3::Client::from_conf(config);

        let bucket_name = load_env("S3_BUCKET")?;
        match client.head_bucket().bucket(&bucket_name).send().await {
            Ok(_) => {
                tracing::info!("Bucket '{bucket_name}' already exists");
            }
            Err(SdkError::ServiceError(e)) if let &HeadBucketError::NotFound(_) = e.err() => {
                tracing::info!("Creating new bucket '{bucket_name}'");
                client.create_bucket().bucket(&bucket_name).send().await?;
            }
            Err(e) => {
                return Err(e.into());
            }
        }

        Ok((client, bucket_name))
    }
}

pub fn load_env(var: &'static str) -> anyhow::Result<String> {
    env::var(var).context(format!("{var} must be set in .env or environment"))
}
