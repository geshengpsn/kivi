use std::{collections::HashMap, net::SocketAddr, path::PathBuf, str::FromStr, sync::{Arc, Mutex}};
use axum::{Router, extract::{ConnectInfo, State, WebSocketUpgrade, ws::{Message, Utf8Bytes, WebSocket}}, http::StatusCode, response::{IntoResponse, Response}, routing::{get, post}, Json};
use tower_http::services::ServeDir;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use futures_util::{SinkExt, StreamExt};

#[derive(Clone)]
struct AppState {
    web_clients: Arc<Mutex<HashMap<usize, mpsc::UnboundedSender<String>>>>,
    registered_apps: Arc<Mutex<Vec<AppInfo>>>,
    next_client_id: Arc<Mutex<usize>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AppInfo {
    name: String,
    address: String,
    port: u16,
}

impl AppState {
    fn new() -> Self {
        Self {
            web_clients: Arc::new(Mutex::new(HashMap::new())),
            registered_apps: Arc::new(Mutex::new(Vec::new())),
            next_client_id: Arc::new(Mutex::new(0)),
        }
    }

    fn add_web_client(&self, sender: mpsc::UnboundedSender<String>) -> usize {
        let mut id = self.next_client_id.lock().unwrap();
        *id += 1;
        let client_id = *id;
        self.web_clients.lock().unwrap().insert(client_id, sender);
        client_id
    }

    fn remove_web_client(&self, id: usize) {
        self.web_clients.lock().unwrap().remove(&id);
    }

    fn register_app(&self, app: AppInfo) {
        self.registered_apps.lock().unwrap().push(app.clone());
        // Broadcast will be handled by the caller
    }

    fn broadcast_apps(&self) {
        let apps = self.registered_apps.lock().unwrap().clone();
        if let Ok(message) = serde_json::to_string(&apps) {
            let mut clients = self.web_clients.lock().unwrap();
            clients.retain(|_, sender| {
                sender.send(message.clone()).is_ok()
            });
        }
    }
}

#[tokio::main]
async fn main() {
    let assets_dir = PathBuf::from_str("./dist").unwrap();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/test", get(|| async { "WebSocket route should be here" }))
        .route("/register", post(register_app))
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .with_state(AppState::new());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, ConnectInfo(addr): ConnectInfo<SocketAddr>, state: State<AppState>) -> Response {
    println!("WebSocket upgrade request from: {}", addr);
    ws.on_upgrade(move |socket| handle_web_client(socket, addr, state))
}

async fn handle_web_client(socket: WebSocket, addr: SocketAddr, state: State<AppState>) {
    println!("Web client connected from: {}", addr);

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let client_id = state.add_web_client(tx);

    let (mut sender, mut receiver) = socket.split();

    // Send current registered apps immediately
    {
        let apps = state.registered_apps.lock().unwrap().clone();
        if let Ok(message) = serde_json::to_string(&apps) {
            let _ = sender.send(Message::Text(Utf8Bytes::from(message))).await;
        }
    }

    // Handle incoming messages from channel and WebSocket
    loop {
        tokio::select! {
            message = rx.recv() => {
                match message {
                    Some(msg) => {
                        if sender.send(Message::Text(Utf8Bytes::from(msg))).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => {
                        println!("Web client {} disconnected", addr);
                        break;
                    }
                    Some(Ok(Message::Ping(ping))) => {
                        let _ = sender.send(Message::Pong(ping)).await;
                    }
                    Some(Ok(_)) => {} // Ignore other messages
                    Some(Err(_)) => break,
                    None => break,
                }
            }
        }
    }

    state.remove_web_client(client_id);
}

#[axum::debug_handler]
async fn register_app(
    state: State<AppState>,
    Json(app_info): Json<AppInfo>,
) -> impl IntoResponse {
    println!("Registering app: {} at {}:{}", app_info.name, app_info.address, app_info.port);
    state.register_app(app_info);
    state.broadcast_apps();
    (StatusCode::OK, "App registered successfully")
}
