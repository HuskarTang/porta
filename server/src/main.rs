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
    /// Path to configuration file (default: user config directory/porta/config.toml)
    #[arg(short, long)]
    config: Option<PathBuf>,

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

    // Determine config path: use provided path or default to user config directory
    let config_path = if let Some(path) = args.config {
        path
    } else {
        Config::default_config_path()?
    };

    // Load configuration, create default if missing
    let (mut config, created) = Config::load_or_create_default(&config_path)?;

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
        println!("Default config created at {}", config_path.display());
        println!("Please review and modify the configuration as needed.");
        println!();
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
    tracing::info!("Config file: {}", config_path.display());
    tracing::info!("Node name: {}", config.node.name);
    tracing::info!("Node role: {}", config.node.role);
    tracing::info!("Database: {}", config.database.path);

    // Set environment variables for the backend to access configuration
    // Note: Backend should ideally use config passed directly, but for now we use env vars
    std::env::set_var("PORTA_ROLE", &config.node.role);
    std::env::set_var("PORTA_DB", &config.database.path);
    std::env::set_var("PORTA_P2P_TCP_PORT", &config.p2p.tcp_port.to_string());
    std::env::set_var("PORTA_NODE_NAME", &config.node.name);
    if let Some(ref key_path) = config.node.key_path {
        std::env::set_var("PORTA_KEY_PATH", key_path);
    }

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
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .with_file(true)
                        .with_line_number(true)
                        .with_target(true)
                )
                .init();
        }
        "pretty" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .pretty()
                        .with_file(true)
                        .with_line_number(true)
                        .with_target(true)
                )
                .init();
        }
        _ => {
            // Default: compact with file and line number
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_file(true)
                        .with_line_number(true)
                        .with_target(true)
                )
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
