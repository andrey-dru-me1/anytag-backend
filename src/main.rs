// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The Anytag Backend Authors

mod db;
mod dto;
mod handlers;
mod models;
mod router;
mod schema;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Establish database connection pool
    let pool = db::establish_connection_pool();
    tracing::info!("Database connection pool established");

    // Create router
    let app = router::create_router(pool);

    // Start server
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
