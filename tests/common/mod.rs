// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

#![allow(dead_code)]

use anyhow::Context;
use anytag_backend::{config, router};
use uuid::Uuid;

pub struct TestApp {
    pub db_pool: config::DbPool,
    router: axum::Router,
}

impl TestApp {
    pub async fn new() -> anyhow::Result<Self> {
        let config = config::AppConfig::from_dotenv()?;
        Self::from_config(config).await
    }

    pub async fn from_config(config: config::AppConfig) -> anyhow::Result<Self> {
        let app_state = config::AppState {
            db_pool: db::setup_test_db_pool(&config.database_url).await?,
            s3_client: s3::mock_s3_client(),
            config,
        };
        Ok(Self {
            db_pool: app_state.db_pool.clone(),
            router: router::create_router(app_state),
        })
    }

    pub async fn with_temporary_s3_bucket<Fut, E>(
        test_logic: impl FnOnce(Self) -> Fut,
    ) -> anyhow::Result<()>
    where
        Fut: Future<Output = Result<(), E>>,
        E: Into<anyhow::Error> + Send + Sync + 'static,
    {
        let config = config::AppConfig::from_dotenv()?;
        Self::from_config_with_temporary_s3_bucket(config, test_logic).await
    }

    pub async fn from_config_with_temporary_s3_bucket<Fut, E>(
        mut config: config::AppConfig,
        test_logic: impl FnOnce(Self) -> Fut,
    ) -> anyhow::Result<()>
    where
        Fut: Future<Output = Result<(), E>>,
        E: Into<anyhow::Error> + Send + Sync + 'static,
    {
        let bucket_name = format!("test-bucket-{}", Uuid::new_v4());
        config.s3.media_bucket_name = bucket_name.clone();
        let app_state = config::AppState {
            db_pool: db::setup_test_db_pool(&config.database_url).await?,
            s3_client: config.s3.build_client().await?,
            config,
        };
        let s3_client = app_state.s3_client.clone();
        let test_app = Self {
            db_pool: app_state.db_pool.clone(),
            router: router::create_router(app_state),
        };

        let result = test_logic(test_app).await;

        s3_client
            .delete_bucket()
            .bucket(&bucket_name)
            .send()
            .await
            .context(format!("Could not delete bucket {}", bucket_name))?;

        result.map_err(Into::into)
    }

    pub fn router(&self) -> axum::Router {
        self.router.clone()
    }
}

mod db {
    use anyhow::Context;
    use anytag_backend::config::DbPool;
    use diesel_async::AsyncConnection;
    use diesel_async::pg::AsyncPgConnection;
    use diesel_async::pooled_connection::deadpool::Pool;
    use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};

    /// Create a test database pool from the `DATABASE_URL` environment variable.
    ///
    /// Uses `max_size=1` to minimise resource usage in tests.
    pub async fn setup_test_db_pool(database_url: &str) -> anyhow::Result<DbPool> {
        let mut manager_config = ManagerConfig::<AsyncPgConnection>::default();
        manager_config.custom_setup = Box::new(|url| {
            Box::pin(async move {
                let mut conn = AsyncPgConnection::establish(url)
                    .await
                    .map_err(|e| diesel::result::ConnectionError::BadConnection(e.to_string()))?;

                conn.begin_test_transaction()
                    .await
                    .map_err(diesel::result::ConnectionError::CouldntSetupConfiguration)?;

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
    pub fn mock_s3_client() -> aws_sdk_s3::Client {
        let s3_config = aws_sdk_s3::config::Builder::new()
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
            .build();
        aws_sdk_s3::Client::from_conf(s3_config)
    }
}
