use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    process::Command,
    sync::mpsc::{self, SendError},
    thread::{sleep, spawn},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{Router, routing::get};
use base64::{Engine, prelude::BASE64_URL_SAFE};
use tokio::sync::oneshot;
use tower_http::services::ServeDir;
use tungstenite::{Bytes, Message, accept};

pub trait LoggableData {
    const TYPE: u16;
    fn size(&self) -> usize;
    fn extend_from_bytes(&self, vec: &mut Vec<u8>);
}

impl<T: LoggableData> LoggableData for &T {
    const TYPE: u16 = T::TYPE;
    fn size(&self) -> usize {
        T::size(self)
    }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        T::extend_from_bytes(self, vec);
    }
}

impl LoggableData for f64 {
    const TYPE: u16 = 0;
    fn size(&self) -> usize {
        8
    }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.to_le_bytes());
    }
}

pub struct Box3 {
    pub size: [f64; 3],
}

impl LoggableData for Box3 {
    const TYPE: u16 = 1;
    fn size(&self) -> usize {
        24
    }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(unsafe {
            std::slice::from_raw_parts(self.size.as_ptr() as *const u8, 24)
        });
    }
}

pub struct BoxLine3 {
    size: [f64; 3],
}

impl LoggableData for BoxLine3 {
    const TYPE: u16 = 2;
    fn size(&self) -> usize {
        24
    }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(unsafe {
            std::slice::from_raw_parts(self.size.as_ptr() as *const u8, 24)
        });
    }
}

pub struct Sphere {
    radius: f64,
}

impl LoggableData for Sphere {
    const TYPE: u16 = 3;
    fn size(&self) -> usize {
        8
    }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.radius.to_le_bytes());
    }
}

pub struct Cylinder {
    radius: f64,
    height: f64,
}

impl LoggableData for Cylinder {
    const TYPE: u16 = 4;
    fn size(&self) -> usize {
        16
    }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.radius.to_le_bytes());
        vec.extend_from_slice(&self.height.to_le_bytes());
    }
}

pub struct Capsule {
    radius: f64,
    height: f64,
}

impl LoggableData for Capsule {
    const TYPE: u16 = 5;
    fn size(&self) -> usize {
        16
    }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.radius.to_le_bytes());
        vec.extend_from_slice(&self.height.to_le_bytes());
    }
}

pub struct Stl {
    data: Vec<u8>,
}

impl LoggableData for Stl {
    const TYPE: u16 = 6;
    fn size(&self) -> usize {
        self.data.len()
    }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        // vec.extend_from_slice(&self.path.len().to_le_bytes());
        vec.extend_from_slice(&self.data);
    }
}

pub struct MeshMaterial {
    color: [u8; 3],
    roughness: f64,
    metalness: f64,
}

impl LoggableData for MeshMaterial {
    const TYPE: u16 = 7;
    fn size(&self) -> usize {
        19
    }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.color);
        vec.extend_from_slice(&self.roughness.to_le_bytes());
        vec.extend_from_slice(&self.metalness.to_le_bytes());
    }
}

impl LoggableData for nalgebra::Matrix4<f64> {
    const TYPE: u16 = 8;
    fn size(&self) -> usize {
        128
    }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(unsafe {
            std::slice::from_raw_parts(self.as_ptr() as *const u8, 128)
        });
    }
}

pub struct Arrow3 {
    start: nalgebra::Vector3<f64>,
    end: nalgebra::Vector3<f64>,
    color: [u8; 3],
}

impl LoggableData for Arrow3 {
    const TYPE: u16 = 9;

    fn size(&self) -> usize {
        51
    }

    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(unsafe {
            std::slice::from_raw_parts(self.start.as_ptr() as *const u8, 24)
        });
        vec.extend_from_slice(unsafe {
            std::slice::from_raw_parts(self.end.as_ptr() as *const u8, 24)
        });
        vec.extend_from_slice(&self.color);
    }
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn run_server(port: u16, close_rx: oneshot::Receiver<()>) {
    let serve_dir = ServeDir::new(".");
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .nest_service("/userfile", serve_dir);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    tokio::select! {
        _ = axum::serve(listener, app) => {
            println!("Server running on port {}", port);
        }
        _ = close_rx => {
            println!("Server stopped");
        }
    }
}

async fn run_dev_server(port: u16, close_rx: oneshot::Receiver<()>) {
    let serve_dir = ServeDir::new(".");
    let app = Router::new()
        // `GET /` goes to `root`
        .nest_service("/userfile", serve_dir);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(async { if close_rx.await.is_ok() {} })
        .await
        .unwrap();
}

pub struct MonitorTab {
    _close_tx: oneshot::Sender<()>,
    tx: mpsc::Sender<Vec<u8>>,
}

impl Default for MonitorTab {
    fn default() -> Self {
        Self::new(9876, 9876, 9877)
    }
}

impl MonitorTab {
    pub fn new(http_port: u16, open_port: u16, ws_port: u16) -> Self {
        let (_close_tx, rx) = tokio::sync::oneshot::channel::<()>();
        spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(run_dev_server(http_port, rx));
        });
        sleep(Duration::from_millis(100));
        Command::new("open")
            .arg(format!("http://localhost:{}/?addr={}", open_port, BASE64_URL_SAFE.encode(ws_port.to_string().as_bytes())))
            .output()
            .unwrap();
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        spawn(move || {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), ws_port);
            let server = TcpListener::bind(addr).unwrap();
            loop {
                let (stream, addr) = loop {
                    if let Ok(a) = server.accept() {
                        break a;
                    }
                };
                println!("WebSocket connection accepted from: {}", addr);
                let mut websocket = accept(stream).expect("Failed to accept WebSocket connection");
                loop {
                    let command = rx.recv();
                    match command {
                        Ok(data) => {
                            if websocket.send(Message::Binary(Bytes::from(data))).is_err() {
                                break;
                            }
                        }
                        // channel is disconnected
                        Err(_) => {
                            websocket.close(None).unwrap();
                            while websocket.flush().is_err() {
                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }
                            return;
                        }
                    }
                }
                println!("WebSocket connection closed");
            }
        });
        MonitorTab { _close_tx, tx }
    }

    pub fn log<T: LoggableData>(&self, path: &str, data: T) -> Result<(), SendError<Vec<u8>>> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_nanos();
        let mut vec = Vec::with_capacity(16 + 2 + path.len() + 2 + data.size());
        vec.extend_from_slice(&timestamp.to_le_bytes());
        vec.extend_from_slice(&(path.len() as u16).to_le_bytes());
        vec.extend_from_slice(path.as_bytes());
        vec.extend_from_slice(&T::TYPE.to_le_bytes());
        data.extend_from_bytes(&mut vec);
        self.tx.send(vec)?;
        Ok(())
    }
}
