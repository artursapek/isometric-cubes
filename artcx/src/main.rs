use axum::{
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }
    let not_found_service = handle_404.into_service();

    let app = Router::new()
        .nest_service("/", ServeFile::new("./static/index.html"))
        .nest_service("/art.pub", ServeFile::new("./static/art.pub"))
        .nest_service(
            "/static",
            ServeDir::new("./static").not_found_service(not_found_service),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
