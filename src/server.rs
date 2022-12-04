//! Server functions for building Instrumentality.
//!
//! We build the tracing, service, router in this module.

use std::net::SocketAddr;

use axum::http::header::{self, HeaderValue};
use axum::http::StatusCode;
use axum::middleware;
use axum::{
    error_handling::HandleErrorLayer,
    extract::Extension,
    routing::{delete, get, post},
    Json, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;
use chrono::Duration;
use tower::ServiceBuilder;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::BoxError;
use tracing_subscriber::{prelude::*, EnvFilter};

use crate::config::IConfig;
use crate::database;
use crate::database::DBHandle;
use crate::database::DBPool;
use crate::routes::add::*;
use crate::routes::create::*;
use crate::routes::default::*;
use crate::routes::frontpage::*;
use crate::routes::halt::*;
use crate::routes::invite::*;
use crate::routes::login::*;
use crate::routes::queue::*;
use crate::routes::register::*;
use crate::routes::reset::*;
use crate::routes::response::ErrorResponse;
use crate::routes::types::*;
use crate::routes::update::*;
use crate::routes::view::*;

pub async fn build_server(
    config: &IConfig,
) -> (Router, RustlsConfig, SocketAddr, Handle) {
    let db_pool = database::open(config).await.unwrap();

    let handle: Handle = Handle::new();

    build_workers(db_pool.handle().await, config.clone()).await;
    tracing::info!("Workers built.");

    let app = build_app(config.clone(), db_pool, handle.clone());
    tracing::info!("Application built.");

    let tls_config = build_tls(&config.tls.cert, &config.tls.key).await;
    tracing::info!("TLS configuration loaded.");

    let addr = build_address(&config.network.address, &config.network.port);

    (app, tls_config, addr, handle)
}

pub fn build_tracing(log_level: &str) {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::new(log_level))
        .init();
}

fn build_app(config: IConfig, db_pool: DBPool, handle: Handle) -> Router {
    let service_builder = ServiceBuilder::new()
        .layer(middleware::from_fn(error_transformer))
        .layer(HandleErrorLayer::new(|error: BoxError| async move {
            if error.is::<tower::timeout::error::Elapsed>() {
                ok!(REQUEST_TIMEOUT, "Request timed out.")
            } else {
                error!(INTERNAL_SERVER_ERROR, "Internal server error.")
            }
        }))
        .layer(Extension(config))
        .layer(Extension(db_pool))
        .layer(Extension(handle))
        .layer(SetResponseHeaderLayer::overriding(
            header::SERVER,
            HeaderValue::from_static("instrumentality"),
        ))
        // .layer(from_extractor::<ContentLengthLimit<(), 10_000_000>>())
        // Need a content length limit, but this breaks integration tests.
        // <()... doesn't remove headers but breaks POSTs and vice versa.
        .timeout(std::time::Duration::from_secs(5));

    Router::new()
        .route("/", get(frontpage))
        .route("/add", post(add))
        .route("/create", post(create))
        .route("/delete", delete(crate::routes::delete::delete))
        .route("/halt", get(halt))
        .route("/invite", get(invite))
        .route("/login", get(login))
        .route("/queue", get(queue))
        .route("/register", post(register))
        .route("/reset", get(reset))
        .route("/types", get(types))
        .route("/update", post(update))
        .route("/view", get(view))
        .layer(service_builder)
        .fallback(default)
}

fn build_address(address: &str, port: &str) -> SocketAddr {
    format!("{address}:{port}").parse().unwrap()
}

async fn build_tls(cert: &str, key: &str) -> RustlsConfig {
    match RustlsConfig::from_pem_file(cert, key).await {
        Ok(tls_config) => tls_config,
        Err(_) => {
            tracing::error!("Failed to create TLS configuration.");
            panic!("Failed to create TLS configuration.")
        }
    }
}

async fn build_workers(mut db: DBHandle, config: IConfig) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            clear_old_locks(
                &mut db,
                Duration::seconds(config.settings.queue_timeout_secs),
            )
            .await;
        }
    });
}
