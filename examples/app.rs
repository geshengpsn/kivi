use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    sync::mpsc::{Sender, channel},
    thread::spawn,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tungstenite::{Bytes, Message, accept};

trait LoggableData {
    const TYPE: u16;
    fn to_bytes(&self) -> &[u8];
}

impl<T: LoggableData> LoggableData for &T {
    const TYPE: u16 = T::TYPE;
    fn to_bytes(&self) -> &[u8] {
        T::to_bytes(self)
    }
}

impl LoggableData for f64 {
    const TYPE: u16 = 0;
    fn to_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const f64 as *const u8, 8) }
    }
}

enum Mesh {
    Box([f64; 3]),
    BoxLine([f64; 3]),
    Sphere(f64),
    Cylinder([f64; 2]),
    Capsule([f64; 2]),
    // file name, path
    Stl(String),
    // file name, path
    // Gltf(String, String),
}

impl LoggableData for Mesh {
    const TYPE: u16 = 1;
    fn to_bytes(&self) -> &[u8] {
        // self.data.as_slice()
        todo!()
    }
}

struct MeshMaterial {
    color: [u8; 3],
    roughness: f64,
    metalness: f64,
}

impl LoggableData for MeshMaterial {
    const TYPE: u16 = 2;

    fn to_bytes(&self) -> &[u8] {
        todo!()
    }
}

impl LoggableData for nalgebra::Matrix4<f64> {
    const TYPE: u16 = 3;
    
    fn to_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, 128) }
    }
}



enum Command {
    Close,
    Data(Vec<u8>),
}

struct Monitor {
    tx: Sender<Command>,
}

impl Drop for Monitor {
    fn drop(&mut self) {
        self.tx.send(Command::Close).unwrap();
    }
}

impl Monitor {
    fn new(port: u16) -> Self {
        let (tx, rx) = channel::<Command>();
        spawn(move || {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
            let server = TcpListener::bind(addr).unwrap();
            let (stream, addr) = loop {
                if let Ok(a) = server.accept() {
                    break a;
                }
            };
            println!("WebSocket connection accepted from: {}", addr);
            let mut websocket = accept(stream).unwrap();
            loop {
                let command = rx.recv();
                match command {
                    Ok(command) => match command {
                        Command::Close => {
                            websocket.close(None).unwrap();
                            return;
                        }
                        Command::Data(data) => {
                            websocket.send(Message::Binary(Bytes::from(data))).unwrap();
                        }
                    },
                    Err(_) => {
                        websocket.close(None).unwrap();
                        return;
                    }
                }
            }
        });
        Self { tx }
    }

    // --- 16 bytes timestamp --- 2 byte path length --- path --- 2 bytes data type --- data ---
    fn log<T: LoggableData>(&self, path: &str, data: T) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_nanos();
        let data_bytes = data.to_bytes();
        let mut vec = Vec::with_capacity(16 + 2 + path.len() + 2 + data_bytes.len());
        vec.extend_from_slice(&timestamp.to_le_bytes());
        vec.extend_from_slice(&(path.len() as u16).to_le_bytes());
        vec.extend_from_slice(path.as_bytes());
        vec.extend_from_slice(&T::TYPE.to_le_bytes());
        vec.extend_from_slice(data_bytes);
        self.tx.send(Command::Data(vec)).unwrap();
    }
}

/// A WebSocket echo server
fn main() {
    let monitor = Monitor::new(9876);
    loop {
        monitor.log("test", 1.0);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
