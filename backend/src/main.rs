mod models;
mod routes;
mod services;

use axum::routing::{get, post};
use axum::Router;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let katas_dir = PathBuf::from("../katas");
    let katas = services::kata_loader::load_all_katas(&katas_dir)
        .expect("Failed to load katas");
    let katas = Arc::new(katas);

    let api = Router::new()
        .route("/katas", get(routes::katas::list_katas))
        .route("/katas/{id}", get(routes::katas::get_kata))
        .route("/playground/run", post(routes::playground::run_code))
        .with_state(katas);

    let app = Router::new()
        .nest("/api", api)
        .layer(CorsLayer::permissive())
        .fallback_service(ServeDir::new("../frontend/dist"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Server running on http://localhost:3000");
    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
