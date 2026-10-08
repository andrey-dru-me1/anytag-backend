// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use std::{env, fmt};

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

pub type DbPool = Pool<AsyncPgConnection>;

#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_ttl_minutes: i64,
    pub refresh_token_ttl_days: i64,
}

impl JwtConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Self {
            secret: load_env("JWT_SECRET")?,
            access_token_ttl_minutes: load_i64_env_or_default("ACCESS_TOKEN_TTL_MINUTES", 15)?,
            refresh_token_ttl_days: load_i64_env_or_default("REFRESH_TOKEN_TTL_DAYS", 30)?,
        }
        .validate()
    }

    fn validate(self) -> anyhow::Result<Self> {
        anyhow::ensure!(
            self.secret.len() >= 32,
            "JWT_SECRET must be at least 32 bytes long"
        );
        anyhow::ensure!(
            self.access_token_ttl_minutes > 0,
            "ACCESS_TOKEN_TTL_MINUTES must be greater than zero"
        );
        anyhow::ensure!(
            self.refresh_token_ttl_days > 0,
            "REFRESH_TOKEN_TTL_DAYS must be greater than zero"
        );

        let refresh_token_ttl_minutes = self
            .refresh_token_ttl_days
            .checked_mul(24 * 60)
            .context("REFRESH_TOKEN_TTL_DAYS is too large")?;
        anyhow::ensure!(
            refresh_token_ttl_minutes > self.access_token_ttl_minutes,
            "refresh token lifetime must be longer than access token lifetime"
        );

        Ok(self)
    }
}

impl fmt::Debug for JwtConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JwtConfig")
            .field("secret", &"[REDACTED]")
            .field("access_token_ttl_minutes", &self.access_token_ttl_minutes)
            .field("refresh_token_ttl_days", &self.refresh_token_ttl_days)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct S3Config {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint_url: String,
    pub media_bucket_name: String,
}

impl S3Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            access_key_id: load_env("AWS_ACCESS_KEY_ID")?,
            secret_access_key: load_env("AWS_SECRET_ACCESS_KEY")?,
            endpoint_url: load_env("S3_BASE_URL")?,
            media_bucket_name: load_env("S3_BUCKET")?,
        })
    }

    /// Provisions the S3 client and ensures the media bucket exists,
    /// creating it when missing.
    pub async fn build_client(&self) -> anyhow::Result<aws_sdk_s3::Client> {
        let credentials = Credentials::new(
            &self.access_key_id,
            &self.secret_access_key,
            None,
            None,
            "manual",
        );

        let region_provider =
            RegionProviderChain::default_provider().or_else(Region::new("us-east-1"));
        let shared_config = aws_config::from_env().region(region_provider).load().await;

        let config = aws_sdk_s3::config::Builder::from(&shared_config)
            .credentials_provider(credentials)
            .endpoint_url(&self.endpoint_url)
            .force_path_style(true)
            .build();
        let client = aws_sdk_s3::Client::from_conf(config);

        let bucket_name = &self.media_bucket_name;
        match client.head_bucket().bucket(bucket_name).send().await {
            Ok(_) => {
                tracing::info!("Bucket '{bucket_name}' already exists");
            }
            Err(SdkError::ServiceError(e)) if let &HeadBucketError::NotFound(_) = e.err() => {
                tracing::info!("Creating new bucket '{bucket_name}'");
                client.create_bucket().bucket(bucket_name).send().await?;
            }
            Err(e) => {
                return Err(e.into());
            }
        }

        Ok(client)
    }
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub s3: S3Config,
    pub jwt: JwtConfig,
    pub database_url: String,
    pub base_url: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<AppConfig> {
        Ok(Self {
            s3: S3Config::from_env()?,
            jwt: JwtConfig::from_env()?,
            database_url: load_env("DATABASE_URL")?,
            base_url: load_env("BASE_URL")?,
        })
    }

    pub fn from_dotenv() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Self::from_env()
    }
}

/// Shared application state handed to Axum via `State<AppState>`.
///
/// Holds runtime resources (database pool, S3 client) plus the immutable
/// [`AppConfig`] settings.
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub s3_client: aws_sdk_s3::Client,
    pub config: AppConfig,
}

impl AppState {
    /// Build the full application state from the environment: loads settings,
    /// creates the database pool and provisions the S3 client (auto-creating
    /// the media bucket on startup).
    pub async fn from_config(config: AppConfig) -> anyhow::Result<AppState> {
        Ok(AppState {
            db_pool: Self::setup_database(&config.database_url)?,
            s3_client: config.s3.build_client().await?,
            config,
        })
    }

    fn setup_database(database_url: &str) -> anyhow::Result<DbPool> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        Pool::builder(config)
            .build()
            .context("Failed to create database connection pool")
    }
}

fn load_env(var: &'static str) -> anyhow::Result<String> {
    env::var(var).context(format!("{var} must be set in .env or environment"))
}

fn load_i64_env_or_default(var: &'static str, default: i64) -> anyhow::Result<i64> {
    env::var(var)
        .unwrap_or_else(|_| default.to_string())
        .parse()
        .context(format!("{var} must be a valid integer"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jwt_config(
        secret: &str,
        access_token_ttl_minutes: i64,
        refresh_token_ttl_days: i64,
    ) -> JwtConfig {
        JwtConfig {
            secret: secret.to_string(),
            access_token_ttl_minutes,
            refresh_token_ttl_days,
        }
    }

    // -----------------------------------------------------------------------
    // JwtConfig::validate
    // -----------------------------------------------------------------------

    #[test]
    fn test_jwt_config_validate_accepts_valid_config() {
        let config = jwt_config("a-secure-test-secret-with-32-bytes", 15, 30);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_jwt_config_validate_rejects_short_secret() {
        let error = jwt_config("short-secret", 15, 30)
            .validate()
            .expect_err("short JWT secret should be rejected");

        assert_eq!(
            error.to_string(),
            "JWT_SECRET must be at least 32 bytes long"
        );
    }

    #[test]
    fn test_jwt_config_validate_rejects_non_positive_access_ttl() {
        let error = jwt_config("a-secure-test-secret-with-32-bytes", 0, 30)
            .validate()
            .expect_err("non-positive access token TTL should be rejected");

        assert_eq!(
            error.to_string(),
            "ACCESS_TOKEN_TTL_MINUTES must be greater than zero"
        );
    }

    #[test]
    fn test_jwt_config_validate_rejects_non_positive_refresh_ttl() {
        let error = jwt_config("a-secure-test-secret-with-32-bytes", 15, 0)
            .validate()
            .expect_err("non-positive refresh token TTL should be rejected");

        assert_eq!(
            error.to_string(),
            "REFRESH_TOKEN_TTL_DAYS must be greater than zero"
        );
    }

    #[test]
    fn test_jwt_config_validate_rejects_refresh_ttl_not_longer_than_access_ttl() {
        let error = jwt_config("a-secure-test-secret-with-32-bytes", 24 * 60, 1)
            .validate()
            .expect_err("refresh token TTL should be longer than access token TTL");

        assert_eq!(
            error.to_string(),
            "refresh token lifetime must be longer than access token lifetime"
        );
    }

    #[test]
    fn test_jwt_config_validate_rejects_refresh_ttl_overflow() {
        let error = jwt_config("a-secure-test-secret-with-32-bytes", 15, i64::MAX)
            .validate()
            .expect_err("overflowing refresh token TTL should be rejected");

        assert_eq!(error.to_string(), "REFRESH_TOKEN_TTL_DAYS is too large");
    }
}
