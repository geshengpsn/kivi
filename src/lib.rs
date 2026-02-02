use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    path::Path,
    process::Command,
    sync::mpsc::{self, SendError},
    thread::{JoinHandle, sleep, spawn},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{Router, routing::get};
use tokio::sync::oneshot;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
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

pub struct NullData;

impl LoggableData for NullData {
    const TYPE: u16 = 0;

    fn size(&self) -> usize {
        0
    }

    fn extend_from_bytes(&self, _vec: &mut Vec<u8>) {
        // do nothing
    }
}

pub struct Stl(pub String);

impl Stl {
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        // check if file exists
        if !path.exists() {
            panic!("File does not exist: {}", path.display());
        }
        // 去掉文件路径前面的"./"
        let path = path.strip_prefix("./").unwrap_or(path);
        Stl(path.to_string_lossy().to_string())
    }
}

impl LoggableData for Stl {
    const TYPE: u16 = 1;
    fn size(&self) -> usize {
        self.0.len()
    }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(self.0.as_bytes());
    }
}

pub struct MeshMaterial {
    pub color: [u8; 3],
    pub roughness: f64,
    pub metalness: f64,
}

impl LoggableData for MeshMaterial {
    const TYPE: u16 = 2;
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
    const TYPE: u16 = 3;
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
    pub start: nalgebra::Vector3<f64>, // 24 bytes
    pub end: nalgebra::Vector3<f64>,   // 24 bytes
    pub color: [u8; 3],
}

impl LoggableData for Arrow3 {
    const TYPE: u16 = 4;

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
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            close_rx.await.ok();
        })
        .await
        .unwrap();
    // println!("Server stopped");
}

async fn run_dev_server(port: u16, close_rx: oneshot::Receiver<()>) {
    let serve_dir = ServeDir::new(".");

    // Configure CORS to allow requests from any origin (for development)
    // You can restrict this to specific origins in production
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // `GET /` goes to `root`
        .nest_service("/userfile", serve_dir)
        // Add CORS layer to handle preflight and CORS headers
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            close_rx.await.ok();
        })
        .await
        .unwrap();
}

fn run_ws_server(port: u16, data_rx: mpsc::Receiver<Vec<u8>>, close_tx: oneshot::Sender<()>) {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let server = TcpListener::bind(addr).unwrap();
    // println!("WebSocket server started");
    loop {
        let (stream, addr) = loop {
            if let Ok(a) = server.accept() {
                break a;
            }
        };
        println!("WebSocket connection accepted from: {}", addr);
        let mut websocket = accept(stream).expect("Failed to accept WebSocket connection");
        loop {
            let msg = data_rx.recv();

            match msg {
                Ok(data) => {
                    if websocket.send(Message::Binary(Bytes::from(data))).is_err() {
                        break;
                    }
                }
                // channel is disconnected
                Err(_) => {
                    // println!("channel disconnected");
                    loop {
                        let can_write = websocket.read();
                        if can_write.is_err() {
                            drop(close_tx);
                            return;
                        }
                    }
                }
            }
        }
    }
}

pub struct MonitorTab {
    job1: JoinHandle<()>,
    job2: JoinHandle<()>,
    tx: mpsc::Sender<Vec<u8>>,
    timeline_nano: u128,
}

impl Default for MonitorTab {
    fn default() -> Self {
        Self::new(9876, 9876, 9877)
    }
}

impl MonitorTab {
    pub fn new(http_port: u16, open_port: u16, ws_port: u16) -> Self {
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let (close_tx, close_rx) = tokio::sync::oneshot::channel::<()>();
        let job1 = spawn(move || run_ws_server(ws_port, rx, close_tx));
        let job2 = spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(run_dev_server(http_port, close_rx));
            // println!("Server stopped");
        });
        // sleep(Duration::from_millis(1000));
        Command::new("open")
            .arg(format!("http://localhost:{}/?addr={}", open_port, ws_port))
            .output()
            .unwrap();
        sleep(Duration::from_millis(100));
        MonitorTab {
            job1,
            job2,
            tx,
            timeline_nano: 0,
        }
    }

    pub fn set_timeline_nano(&mut self, timestamp: u128) {
        self.timeline_nano = timestamp;
    }

    pub fn log<T: LoggableData>(&self, path: &str, data: T) -> Result<(), SendError<Vec<u8>>> {
        // let timestamp = SystemTime::now()
        //     .duration_since(UNIX_EPOCH)
        //     .unwrap_or(Duration::from_secs(0))
        //     .as_nanos();
        let mut vec = Vec::with_capacity(16 + 2 + path.len() + 2 + data.size());
        vec.extend_from_slice(&self.timeline_nano.to_le_bytes());
        vec.extend_from_slice(&(path.len() as u16).to_le_bytes());
        vec.extend_from_slice(path.as_bytes());
        vec.extend_from_slice(&T::TYPE.to_le_bytes());
        data.extend_from_bytes(&mut vec);
        self.tx.send(vec)?;
        Ok(())
    }

    pub fn wait_tab_close(self) {
        drop(self.tx);
        self.job1.join().unwrap();
        self.job2.join().unwrap();
    }
}
