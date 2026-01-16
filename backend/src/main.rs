use porta_backend::create_app;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = create_app().await;
    let listener = TcpListener::bind("0.0.0.0:8090")
        .await
        .expect("bind 8090");
    axum::serve(listener, app).await.expect("server failed");
}
