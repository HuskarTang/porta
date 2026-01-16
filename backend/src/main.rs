use porta_backend::create_app;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "porta_backend=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Porta backend server...");
    
    let app = create_app().await;
    let listener = TcpListener::bind("0.0.0.0:8090")
        .await
        .expect("bind 8090");
    
    tracing::info!("Listening on http://0.0.0.0:8090");
    axum::serve(listener, app).await.expect("server failed");
}
