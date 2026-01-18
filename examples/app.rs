use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    sync::mpsc::{Sender, channel},
    thread::{spawn},
};
use tungstenite::{Bytes, Message, accept};

// pub enum Data {
//     String(String),
//     F64(f64),
//     VecF64(Vec<f64>),
//     F32(f32),
//     VecF32(Vec<f32>),
//     Mat4F64([f64; 16]),
// }

// impl Data {
//     fn to_bytes(&self) -> Bytes {
//         match self {
//             Data::String(s) => Bytes::from(s),
//             Data::F64(f) => Bytes::from(f.to_string()),
//             Data::VecF64(v) => Bytes::from(v.to_string()),
//             Data::F32(f) => Bytes::from(f.to_string()),
//             Data::VecF32(v) => Bytes::from(v.to_string()),
//             Data::Mat4F64(m) => Bytes::from(m.to_string()),
//         }
//     }
// }

trait LoggableData {
    fn to_bytes(&self) -> Bytes;
}

enum Command {
    Close,
    Data(Bytes),
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
                            return
                        }
                        Command::Data(data) => {
                            websocket.send(Message::Binary(data)).unwrap();
                        }
                    },
                    Err(_) => {
                        websocket.close(None).unwrap();
                        return
                    },
                }
            }
        });
        Self { tx }
    }

    fn send(&self, data: Bytes) {
        self.tx.send(Command::Data(data)).unwrap();
    }

    fn log(&self, path: &str, data: impl LoggableData) {
        // let data_bytes = data.to_bytes();
        // let mut bytes = Bytes::new();
        // bytes = bytes + path.as_bytes();
        // bytes.extend(path.as_bytes());
        // bytes.extend(data_bytes);
        // self.send(bytes);
    }
}

/// A WebSocket echo server
fn main() {
    let monitor = Monitor::new(9876);
    loop {
        monitor.send(Bytes::from_static(&[1, 2, 3, 4, 5]));
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
