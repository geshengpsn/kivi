use axum::{
    Json, Router, extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post}
};

use base64::prelude::*;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let port: u16 = 3000;
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/userfile/{filename}", get(get_userfile))
        .route("/opentab", post(open_tab))
        .with_state(port);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

#[axum::debug_handler]
async fn get_userfile(Path(filename): Path<String>) -> impl IntoResponse {
    println!("filename: {}", filename);

    // Read the file
    match tokio::fs::read(&filename).await {
        Ok(contents) => {
            // Return the file contents with 200 OK status
            (StatusCode::OK, contents).into_response()
        }
        Err(e) => {
            // Return 404 if file not found, 500 for other errors
            let status = if e.kind() == std::io::ErrorKind::NotFound {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (status, format!("Error reading file: {}", e)).into_response()
        }
    }
}

#[derive(Deserialize)]
struct OpenTab {
    ws_addr: String,
    path: String,
}

#[axum::debug_handler]
async fn open_tab(State(port): State<u16>, Json(payload): Json<OpenTab>) -> StatusCode {
    println!("ws_addr: {}", payload.ws_addr);
    println!("path: {}", payload.path);
    tokio::process::Command::new("open")
        .arg(format!("http://localhost:{}/?addr={}", port, BASE64_URL_SAFE.encode(payload.ws_addr.as_bytes())))
        .spawn()
        .unwrap();
    StatusCode::OK
}