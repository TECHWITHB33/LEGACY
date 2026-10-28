//! LEGACY Backend: Student Identity Card System with Invoice Liquidity Network
//!
//! Rust backend built with Actix-web providing:
//! - JWT-based authentication and authorization
//! - Cryptographic credential issuance and verification
//! - Soroban smart contract escrow for invoice liquidity
//! - Invoice lifecycle management
//! - Global advancement service for cross-border settlement

use actix_web::{web, App, HttpServer, middleware::Logger};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tracing_subscriber;

use id_card_system::config::Config;
use id_card_system::handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::from_default_env()
                .add_directive("id_card_system=debug".parse().unwrap()),
        )
        .init();

    tracing::info!("Starting LEGACY backend server");

    // Load configuration from environment
    let config = Config::from_env();
    tracing::info!(
        "Config loaded: host={}, port={}, database={}",
        config.host, config.port, config.database_url
    );

    // Initialize database connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database initialized successfully");

    let pool_data = web::Data::new(Arc::new(pool));
    let config_data = web::Data::new(Arc::new(config.clone()));

    let host = config.host.clone();
    let port = config.port;

    tracing::info!(
        "Starting server: http://{}:{}",
        host, port
    );

    // Create HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .app_data(config_data.clone())
            .wrap(Logger::default())
            // Health check
            .route("/health", web::get().to(handlers::health))
            // Invoice & Escrow API (FEAT-004)
            .service(
                web::scope("/api/v1/escrow")
                    .route("/invoices", web::post().to(handlers::escrow::create_invoice))
                    .route("/invoices", web::get().to(handlers::escrow::list_invoices))
                    .route("/invoices/{id}", web::get().to(handlers::escrow::get_invoice))
                    .route("/invoices/{id}/submit", web::post().to(handlers::escrow::submit_invoice))
                    .route("/invoices/{id}/fund", web::post().to(handlers::escrow::fund_invoice))
                    .route("/invoices/{id}/settle", web::post().to(handlers::escrow::settle_invoice))
                    .route("/contracts/{contract_id}", web::get().to(handlers::escrow::get_escrow_status))
            )
    })
    .bind((host.clone(), port))?
    .run()
    .await
}
