// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tokio::sync::oneshot;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "porta_backend=info,porta_tauri=info".into()),
        )
        .init();

    tracing::info!("Starting Porta application...");

    // Create a tokio runtime for the backend
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    // Channel to signal backend readiness
    let (tx, rx) = oneshot::channel::<()>();

    // Spawn the backend server in a separate thread
    let backend_handle = std::thread::spawn(move || {
        runtime.block_on(async {
            tracing::info!("Starting Porta backend server...");
            
            let app = porta_backend::create_app().await;
            let listener = tokio::net::TcpListener::bind("127.0.0.1:8090")
                .await
                .expect("Failed to bind to port 8090");
            
            tracing::info!("Backend server listening on http://127.0.0.1:8090");
            
            // Signal that the backend is ready
            let _ = tx.send(());
            
            axum::serve(listener, app).await.expect("Server failed");
        });
    });

    // Wait for backend to be ready (with timeout)
    std::thread::spawn(move || {
        let _ = rx.blocking_recv();
        tracing::info!("Backend server is ready");
    });

    // Give the backend a moment to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Run the Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_backend_status])
        .run(tauri::generate_context!())
        .expect("Failed to run Tauri application");

    // Wait for backend thread to finish (this will block until app closes)
    let _ = backend_handle.join();
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
