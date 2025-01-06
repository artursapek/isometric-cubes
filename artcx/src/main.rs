use axum::{
    routing::get,
    extract::Path,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, header},
    response::IntoResponse,
    Router,
};
use std::{fs, net::SocketAddr};

use tower_http::{
    services::{ServeDir, ServeFile},
};

#[tokio::main]
async fn main() {
    async fn handle_404() -> (StatusCode, &'static str) {
        (StatusCode::NOT_FOUND, "Not found")
    }
    let not_found_service = handle_404.into_service();

    let blog_service = Router::new()
        .route("/", get(serve_blog_index))
        .route("/:slug", get(serve_blog));

    let app = Router::new()
        .nest_service("/", ServeFile::new("./static/index.html"))
        .nest_service("/blog", blog_service)
        .nest_service("/art.pub", ServeFile::new("./static/art.pub"))
        .nest_service(
            "/static",
            ServeDir::new("./static").not_found_service(not_found_service),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_blog(Path(slug): Path<String>) -> impl IntoResponse {
    // Construct the file path
    let file_path = format!("./static/blog/{}.html", slug);

    // Attempt to read the file
    match fs::read_to_string(&file_path) {
        Ok(contents) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html")],
            contents,
        )
            .into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "404: File not found".to_string(),
        )
            .into_response(),
    }
}

async fn serve_blog_index() -> impl IntoResponse {
    // Construct the file path
    let file_path = "./static/blog/index.html";

    // Attempt to read the file
    match fs::read_to_string(&file_path) {
        Ok(contents) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html")],
            contents,
        )
            .into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "404: File not found".to_string(),
        )
            .into_response(),
    }
}
