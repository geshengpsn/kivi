use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    sync::mpsc::{Sender, channel, SendError},
    thread::spawn,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tungstenite::{Bytes, Message, accept};

trait LoggableData {
    const TYPE: u16;
    fn size(&self) -> usize;
    fn extend_from_bytes(&self, vec: &mut Vec<u8>);
}

impl<T: LoggableData> LoggableData for &T {
    const TYPE: u16 = T::TYPE;
    fn size(&self) -> usize { T::size(self) }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        T::extend_from_bytes(self, vec);
    }
}

impl LoggableData for f64 {
    const TYPE: u16 = 0;
    fn size(&self) -> usize { 8 }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.to_le_bytes());
    }
}

struct Box3 {
    size: [f64; 3],
}

impl LoggableData for Box3 {
    const TYPE: u16 = 1;
    fn size(&self) -> usize { 24 }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(unsafe {
            std::slice::from_raw_parts(self.size.as_ptr() as *const u8, 24)
        });
    }
}

struct BoxLine3 {
    size: [f64; 3],
}

impl LoggableData for BoxLine3 {
    const TYPE: u16 = 2;
    fn size(&self) -> usize { 24 }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(unsafe {
            std::slice::from_raw_parts(self.size.as_ptr() as *const u8, 24)
        });
    }
}

struct Sphere {
    radius: f64,
}

impl LoggableData for Sphere {
    const TYPE: u16 = 3;
    fn size(&self) -> usize { 8 }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.radius.to_le_bytes());
    }
}

struct Cylinder {
    radius: f64,
    height: f64,
}

impl LoggableData for Cylinder {
    const TYPE: u16 = 4;
    fn size(&self) -> usize { 16 }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.radius.to_le_bytes());
        vec.extend_from_slice(&self.height.to_le_bytes());
    }
}

struct Capsule {
    radius: f64,
    height: f64,
}

impl LoggableData for Capsule {
    const TYPE: u16 = 5;
    fn size(&self) -> usize { 16 }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&self.radius.to_le_bytes());
        vec.extend_from_slice(&self.height.to_le_bytes());
    }
}

struct Stl {
    data: Vec<u8>,
}

impl LoggableData for Stl {
    const TYPE: u16 = 6;
    fn size(&self) -> usize { self.data.len() }
    fn extend_from_bytes(&self, vec: &mut Vec<u8>) {
        // vec.extend_from_slice(&self.path.len().to_le_bytes());
        vec.extend_from_slice(&self.data);
    }
}

struct MeshMaterial {
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

struct Arrow3 {
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
                        Ok(command) => match command {
                            Command::Close => {
                                if websocket.close(None).is_err() {
                                    break;
                                }
                                while websocket.flush().is_err() {
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                }
                                break;
                            }
                            Command::Data(data) => {
                                if websocket.send(Message::Binary(Bytes::from(data))).is_err() {
                                    break;
                                }
                            }
                        },
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
        Self { tx }
    }

    // --- 16 bytes timestamp --- 2 byte path length --- path --- 2 bytes data type --- data ---
    fn log<T: LoggableData>(&self, path: &str, data: T) -> Result<(), SendError<Command>> {
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
        self.tx.send(Command::Data(vec))?;
        Ok(())
    }
}

/// A WebSocket echo server
fn main() {
    let monitor = Monitor::new(9876);
    for _ in 0..10 {
        monitor.log("test", Box3 { size: [1.0, 2.0, 3.0] }).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
