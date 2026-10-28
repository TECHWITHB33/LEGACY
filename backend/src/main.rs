mod config;
mod crypto;
mod database;
mod error;
mod handlers;
mod middleware;
mod models;
mod utils;
mod validators;
mod credential_status;
mod credential_versioning;
mod credential_revocation;
mod credential_expiration;
mod qr_verification;
mod biometric_auth;
mod payment_gateway;
mod blockchain;
mod stellar;
mod stellar_payment;
mod jwt;
mod audit;
mod contracts;
mod services;
mod invoice_lifecycle;
mod global_advancement;

use actix_web::{web, App, HttpServer, middleware as actix_middleware};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tracing_subscriber::filter::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment variables
    dotenv::dotenv().ok();
    
    // Setup logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("id_card_system=debug".parse().unwrap())
        )
        .json()
        .init();

    tracing::info!("Starting ID Card System Server...");

    // Load configuration
    let config = config::Config::from_env().expect("Failed to load configuration");
    tracing::info!("Configuration loaded: {:?}", config);

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/id_card_system.db".to_string());
    
    tracing::info!("Connecting to database: {}", database_url);
    
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Database migrations completed successfully");

    let pool = Arc::new(pool);
    let config = Arc::new(config);

    // Server configuration
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid u16");

    tracing::info!("Starting HTTP server on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(actix_middleware::Logger::default())
            .wrap(middleware::CorsMiddleware)
            // Health check (public)
            .route("/health", web::get().to(handlers::health))
            // Auth routes (public)
            .service(
                web::scope("/api/v1/auth")
                    .route("/register", web::post().to(handlers::auth::register))
                    .route("/login", web::post().to(handlers::auth::login))
                    .route("/verify", web::post().to(handlers::auth::verify_signature))
                    .route("/refresh", web::post().to(handlers::auth::refresh_token))
            )
            // Protected routes with JWT middleware
            .service(
                web::scope("/api/v1/credentials")
                    .wrap(middleware::JwtMiddleware)
                    .route("", web::get().to(handlers::credentials::list_credentials))
                    .route("/{id}", web::get().to(handlers::credentials::get_credential))
                    .route("", web::post().to(handlers::credentials::create_credential))
                    .route("/{id}/verify", web::post().to(handlers::credentials::verify_credential))
                    .route("/{id}/revoke", web::post().to(handlers::credentials::revoke_credential))
            )
            // Blockchain routes (public)
            .service(
                web::scope("/api/v1/blockchain")
                    .route("/chain", web::get().to(handlers::blockchain::get_chain))
                    .route("/blocks/{id}", web::get().to(handlers::blockchain::get_block))
                    .route("/validate", web::post().to(handlers::blockchain::validate_chain))
            )
            // QR routes (public for verification)
            .service(
                web::scope("/api/v1/qr")
                    .route("/generate", web::post().to(handlers::qr::generate_qr))
                    .route("/verify", web::post().to(handlers::qr::verify_qr))
            )
            // Payment routes (protected)
            .service(
                web::scope("/api/v1/payments")
                    .wrap(middleware::JwtMiddleware)
                    .route("", web::get().to(handlers::payment::list_payments))
                    .route("", web::post().to(handlers::payment::create_payment))
                    .route("/{id}", web::get().to(handlers::payment::get_payment))
                    .route("/{id}/verify", web::post().to(handlers::payment::verify_payment))
                    .route("/merchant/qr", web::post().to(handlers::payment::generate_merchant_qr))
            )
            // Biometric/WebAuthn routes (protected)
            .service(
                web::scope("/api/v1/biometric")
                    .wrap(middleware::JwtMiddleware)
                    .route("/register/start", web::post().to(handlers::biometric::register_start))
                    .route("/register/complete", web::post().to(handlers::biometric::register_complete))
                    .route("/authenticate/start", web::post().to(handlers::biometric::authenticate_start))
                    .route("/authenticate/complete", web::post().to(handlers::biometric::authenticate_complete))
            )
            // Admin routes (protected)
            .service(
                web::scope("/api/v1/admin")
                    .wrap(middleware::JwtMiddleware)
                    .route("/audit-log", web::get().to(handlers::admin::get_audit_log))
                    .route("/stats", web::get().to(handlers::admin::get_stats))
                    .route("/users", web::get().to(handlers::admin::list_users))
            )
            // Escrow and Invoice routes (protected)
            .service(
                web::scope("/api/v1/escrow")
                    .wrap(middleware::JwtMiddleware)
                    .route("/invoices", web::post().to(handlers::escrow::create_invoice))
                    .route("/invoices/{id}/submit", web::post().to(handlers::escrow::submit_invoice))
                    .route("/invoices/{id}/fund", web::post().to(handlers::escrow::fund_invoice))
                    .route("/invoices/{id}", web::get().to(handlers::escrow::get_invoice))
                    .route("/invoices", web::get().to(handlers::escrow::list_invoices))
                    .route("/invoices/{id}/settle", web::post().to(handlers::escrow::settle_invoice))
                    .route("/contracts/{contract_id}", web::get().to(handlers::escrow::get_escrow_status))
            )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
