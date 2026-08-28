// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use std::{assert_matches, env};

use anytag_backend::config;
use aws_sdk_s3::{error::SdkError, operation::head_bucket::HeadBucketError};

#[tokio::test]
async fn test_ok() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let s3_config = config::S3Config::from_env()?;
    s3_config.build_client().await.map(|_| ())
}

#[tokio::test]
async fn test_wrong_s3_url() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let s3_config = config::S3Config {
        access_key_id: env::var("AWS_ACCESS_KEY_ID")?,
        secret_access_key: env::var("AWS_SECRET_ACCESS_KEY")?,
        endpoint_url: "http://wrong.url".to_string(),
        media_bucket_name: env::var("S3_BUCKET")?,
    };
    let result = s3_config.build_client().await;
    let e = result.unwrap_err();
    let e: SdkError<HeadBucketError> = e.downcast()?;
    assert_matches!(e, SdkError::DispatchFailure(e) if e.is_io());
    Ok(())
}

#[tokio::test]
async fn test_wrong_aws_access_key() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let s3_config = config::S3Config {
        access_key_id: env::var("AWS_ACCESS_KEY_ID")?,
        secret_access_key: "wrong_aws_access_key".to_string(),
        endpoint_url: env::var("S3_BASE_URL")?,
        media_bucket_name: env::var("S3_BUCKET")?,
    };
    let result = s3_config.build_client().await;
    let e = result.unwrap_err();
    let e: SdkError<HeadBucketError> = e.downcast()?;
    assert_matches!(e, SdkError::ServiceError(e) if e.raw().status().as_u16() == 403);
    Ok(())
}
