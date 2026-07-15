// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

use anyhow::Context;
use anytag_backend::{db, router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Establish database connection pool
    let pool = db::establish_connection_pool()?;
    tracing::info!("Database connection pool established");

    // Create router
    let app = router::create_router(pool);

    // Start server
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context(format!("Failed to bind an address {addr}"))?;
    tracing::info!("Server listening on {}", addr);
    axum::serve(listener, app)
        .await
        .context("Failed to run an axum server")?;

    Ok(())
}
