use std::net::SocketAddr;

use axum::{
    Router,
    extract::{ConnectInfo, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
    routing::{any, post},
};
use tokio::process::Command;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

#[tokio::main]
async fn main() {
    let serve_dir = ServeDir::new(".");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest_service("/", serve_dir)
        .route("/ws", any(ws_handler))
        .route("/opentab", post(open_tab_handler))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", 9876))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap()
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr) {
    println!("WebSocket connection accepted from: {addr}");
    loop {
        let msg = socket.recv().await;
        match msg {
            Some(_) => todo!(),
            None => todo!(),
        }
    }
}

async fn open_tab_handler() -> impl IntoResponse {
    Command::new("open")
        .arg(format!("http://localhost:{}/", 9876)).output()
        .await
        .unwrap();
}