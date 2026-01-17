// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;
use tauri::{async_runtime, Manager};

fn init_panic_log() {
    let log_path = std::env::temp_dir().join("porta-tauri-panic.log");
    let _ = std::fs::remove_file(&log_path);
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = writeln!(file, "panic: {}", info);
            let bt = std::backtrace::Backtrace::capture();
            let _ = writeln!(file, "{:?}", bt);
        }
    }));
}

fn main() {
    init_panic_log();
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "porta_backend=info,porta_tauri=info".into()),
        )
        .init();

    tracing::info!("Starting Porta application...");

    // Run the Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_backend_status])
        .setup(|app| {
            // Use app data dir for the embedded backend database and config
            let data_dir = match app.path().app_data_dir() {
                Ok(dir) => dir,
                Err(err) => {
                    tracing::error!("Failed to resolve app data dir: {}", err);
                    std::env::current_dir().unwrap_or_else(|_| ".".into())
                }
            };
            if let Err(err) = std::fs::create_dir_all(&data_dir) {
                tracing::error!(
                    "Failed to create app data dir {}: {}",
                    data_dir.display(),
                    err
                );
            }

            // Set database path and role for backend
            let db_path = data_dir.join("porta.db");
            std::env::set_var("PORTA_DB", db_path.to_string_lossy().to_string());
            std::env::set_var("PORTA_ROLE", "edge");
            // Set default P2P TCP port for desktop app
            std::env::set_var("PORTA_P2P_TCP_PORT", "9000");

            tracing::info!("Database path: {}", db_path.display());
            tracing::info!("Node role: edge");

            // Start backend server asynchronously
            async_runtime::spawn(async move {
                tracing::info!("Starting Porta backend server...");
                let app = porta_backend::create_app().await;
                match tokio::net::TcpListener::bind("127.0.0.1:8090").await {
                    Ok(listener) => {
                        tracing::info!("Backend server listening on http://127.0.0.1:8090");
                        if let Err(err) = axum::serve(listener, app).await {
                            tracing::error!("Backend server failed: {}", err);
                        }
                    }
                    Err(err) => {
                        tracing::error!("Failed to bind backend port 8090: {}", err);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Failed to run Tauri application");
}

/// Command to check if the backend is running
#[tauri::command]
fn get_backend_status() -> Result<String, String> {
    // Try to connect to the backend
    match std::net::TcpStream::connect_timeout(
        &"127.0.0.1:8090".parse().unwrap(),
        std::time::Duration::from_secs(1),
    ) {
        Ok(_) => Ok("running".to_string()),
        Err(_) => Err("Backend not responding".to_string()),
    }
}
