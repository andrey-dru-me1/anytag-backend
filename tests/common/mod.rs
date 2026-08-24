// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

#![allow(dead_code)]

// todo: clean the mess up

use anyhow::Context;
use anytag_backend::{config, router};

pub struct TestApp {
    pub db_pool: config::DbPool,
    router: axum::Router,
}

impl TestApp {
    pub async fn new() -> anyhow::Result<Self> {
        let config = config::Config {
            db_pool: db::setup_test_db_pool().await?,
            s3_client: s3::mock_s3_client(),
            s3_media_bucket: String::new(),
            base_url: String::new(),
        };
        Ok(Self {
            db_pool: config.db_pool.clone(),
            router: router::create_router(config),
        })
    }

    pub async fn with_temporary_s3_bucket<Fut, E>(
        test_logic: impl FnOnce(Self) -> Fut,
    ) -> anyhow::Result<()>
    where
        Fut: Future<Output = Result<(), E>>,
        E: Into<anyhow::Error> + Send + Sync + 'static,
    {
        let s3_test_client = s3::S3TestClient::from_env().await?;
        let config = config::Config {
            db_pool: db::setup_test_db_pool().await?,
            s3_client: s3_test_client.client.clone(),
            s3_media_bucket: s3_test_client.bucket_name.clone(),
            base_url: String::new(),
        };
        let test_app = Self {
            db_pool: config.db_pool.clone(),
            router: router::create_router(config),
        };

        let result = test_logic(test_app).await;

        s3_test_client
            .client
            .delete_bucket()
            .bucket(&s3_test_client.bucket_name)
            .send()
            .await
            .context(format!(
                "Could not delete bucket {}",
                s3_test_client.bucket_name
            ))?;

        result.map_err(Into::into)
    }

    pub fn router(&self) -> axum::Router {
        self.router.clone()
    }
}

mod db {
    use anyhow::Context;
    use anytag_backend::config::{DbPool, load_env};
    use diesel_async::AsyncConnection;
    use diesel_async::pg::AsyncPgConnection;
    use diesel_async::pooled_connection::deadpool::Pool;
    use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};

    /// Create a test database pool from the `DATABASE_URL` environment variable.
    ///
    /// Uses `max_size=1` to minimise resource usage in tests.
    pub async fn setup_test_db_pool() -> anyhow::Result<DbPool> {
        let database_url = load_env("DATABASE_URL")?;

        let mut manager_config = ManagerConfig::<AsyncPgConnection>::default();
        manager_config.custom_setup = Box::new(|url| {
            Box::pin(async move {
                let mut conn = AsyncPgConnection::establish(url)
                    .await
                    .map_err(|e| diesel::result::ConnectionError::BadConnection(e.to_string()))?;

                conn.begin_test_transaction()
                    .await
                    .map_err(|e| diesel::result::ConnectionError::CouldntSetupConfiguration(e))?;

                Ok(conn)
            })
        });

        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
            database_url,
            manager_config,
        );
        Pool::builder(config)
            .max_size(1)
            .build()
            .context("Failed to create test database pool")
    }
}

mod s3 {
    use anyhow::Context;
    use anytag_backend::config::load_env;
    use aws_config::meta::region::RegionProviderChain;
    use aws_sdk_s3::config::{Credentials, Region};
    use uuid::Uuid;

    pub fn mock_s3_client() -> aws_sdk_s3::Client {
        let s3_config = aws_sdk_s3::config::Builder::new()
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .build();
        aws_sdk_s3::Client::from_conf(s3_config)
    }

    pub struct S3TestClient {
        pub client: aws_sdk_s3::Client,
        pub bucket_name: String,
    }

    impl S3TestClient {
        /// Build an S3 client pointing at the local docker-compose SeaweedFS endpoint.
        ///
        /// Reads `S3_BASE_URL`, `S3_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` from the
        /// environment when set, otherwise falls back to the docker-compose defaults.
        /// Ensures the configured bucket exists before returning.
        ///
        /// this module (posts/tags/users) never use the S3 helpers.
        pub async fn from_env() -> anyhow::Result<Self> {
            let base_url = load_env("S3_BASE_URL")?;
            let access_key_id = load_env("AWS_ACCESS_KEY_ID")?;
            let secret_access_key = load_env("AWS_SECRET_ACCESS_KEY")?;
            let bucket = format!("test-bucket-{}", Uuid::new_v4());

            let credentials =
                Credentials::new(&access_key_id, &secret_access_key, None, None, "manual");
            let region_provider =
                RegionProviderChain::default_provider().or_else(Region::new("us-east-1"));
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

            Ok(Self {
                client,
                bucket_name: bucket,
            })
        }
    }
}
