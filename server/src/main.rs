//! Porta Server - Headless P2P Network Service
//!
//! This is the server/service mode binary for Porta, designed for running
//! on servers without a GUI. It reads configuration from a TOML file and
//! runs the P2P backend with HTTP API.

mod config;

use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::Response,
    routing::get,
};
use clap::Parser;
use config::Config;
use include_dir::{include_dir, Dir};
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static WEB_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist");

/// Porta P2P Network Server
#[derive(Parser, Debug)]
#[command(name = "porta-server")]
#[command(author = "Porta Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Porta P2P Network Service - headless server mode")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "porta.toml")]
    config: PathBuf,

    /// Override listen address
    #[arg(long)]
    listen: Option<String>,

    /// Override listen port
    #[arg(short, long)]
    port: Option<u16>,

    /// Override log level
    #[arg(long)]
    log_level: Option<String>,

    /// Print configuration and exit
    #[arg(long)]
    print_config: bool,

    /// Validate configuration and exit
    #[arg(long)]
    validate: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load configuration, create default if missing
    let (mut config, created) = Config::load_or_create_default(&args.config)?;

    // Apply command-line overrides
    if let Some(listen) = args.listen {
        config.server.listen_addr = listen;
    }
    if let Some(port) = args.port {
        config.server.port = port;
    }
    if let Some(level) = args.log_level {
        config.logging.level = level;
    }

    // Validate configuration
    config.validate()?;

    // Ensure database file exists (prevents sqlite open errors)
    config.ensure_db_file()?;

    if created {
        println!("Default config created at {}", args.config.display());
    }

    // Handle --print-config
    if args.print_config {
        println!("{}", toml::to_string_pretty(&config)?);
        return Ok(());
    }

    // Handle --validate
    if args.validate {
        println!("Configuration is valid.");
        return Ok(());
    }

    // Initialize logging
    init_logging(&config)?;

    tracing::info!("Starting Porta server v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Config file: {}", args.config.display());
    tracing::info!("Node name: {}", config.node.name);
    tracing::info!("Node role: {}", config.node.role);

    // Set environment variables for the backend
    std::env::set_var("PORTA_ROLE", &config.node.role);
    std::env::set_var("PORTA_DB", &config.database.path);

    // Create the application (API + embedded web UI)
    let app = porta_backend::create_app()
        .await
        .fallback(get(serve_embedded));

    // Bind to address
    let bind_addr = config.bind_addr();
    let listener = TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", bind_addr, e))?;

    tracing::info!("HTTP API listening on http://{}", bind_addr);
    tracing::info!("P2P TCP port: {}", config.p2p.tcp_port);
    tracing::info!("P2P QUIC port: {}", config.p2p.quic_port);

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

async fn serve_embedded(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = WEB_DIR.get_file(path) {
        return file_response(file.contents(), path);
    }

    // SPA fallback to index.html if asset not found
    if let Some(index) = WEB_DIR.get_file("index.html") {
        return file_response(index.contents(), "index.html");
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap()
}

fn file_response(contents: &[u8], path: &str) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(contents.to_vec()))
        .unwrap()
}

/// Initialize the logging subsystem based on configuration
fn init_logging(config: &Config) -> anyhow::Result<()> {
    let log_level = &config.logging.level;
    let filter = format!("porta_backend={},porta_server={},tower_http=debug", log_level, log_level);

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| filter.into());

    match config.logging.format.as_str() {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        }
        "pretty" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        }
        _ => {
            // Default: compact
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer())
                .init();
        }
    }

    Ok(())
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, initiating graceful shutdown...");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown...");
        }
    }
}
