// use std::{
//     collections::HashMap, net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener}, sync::mpsc::{SendError, Sender, channel}, thread::spawn, time::{Duration, SystemTime, UNIX_EPOCH}
// };
// use tungstenite::{Bytes, Message, accept};

// use kivi::LoggableData;

// enum Command {
//     Close,
//     Data(Vec<u8>),
// }

// struct Monitor {
//     tx: Sender<Command>,
// }

// impl Drop for Monitor {
//     fn drop(&mut self) {
//         self.tx.send(Command::Close).unwrap();
//     }
// }

// impl Monitor {
//     fn new(port: u16) -> Result<Self, reqwest::Error> {
//         let (tx, rx) = channel::<Command>();
//         let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
//         let server = TcpListener::bind(addr).unwrap();
//         spawn(move || {
//             tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(future)
//         });

//         let client = reqwest::blocking::Client::new();
//         let mut map = HashMap::new();
//         map.insert("ws_addr", "ws://localhost:9876/ws");
//         map.insert("path", "test");
//         client.post("http://localhost:3000/opentab").json(&map).send()?;
        
//         spawn(move || {
//             let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
//             let server = TcpListener::bind(addr).unwrap();
//             loop {
//                 let (stream, addr) = loop {
//                     if let Ok(a) = server.accept() {
//                         break a;
//                     }
//                 };
//                 println!("WebSocket connection accepted from: {}", addr);
//                 let mut websocket = accept(stream).expect("Failed to accept WebSocket connection");
//                 loop {
//                     let command = rx.recv();
//                     match command {
//                         Ok(command) => match command {
//                             Command::Close => {
//                                 if websocket.close(None).is_err() {
//                                     break;
//                                 }
//                                 while websocket.flush().is_err() {
//                                     std::thread::sleep(std::time::Duration::from_millis(100));
//                                 }
//                                 break;
//                             }
//                             Command::Data(data) => {
//                                 if websocket.send(Message::Binary(Bytes::from(data))).is_err() {
//                                     break;
//                                 }
//                             }
//                         },
//                         // channel is disconnected
//                         Err(_) => {
//                             websocket.close(None).unwrap();
//                             while websocket.flush().is_err() {
//                                 std::thread::sleep(std::time::Duration::from_millis(100));
//                             }
//                             return;
//                         }
//                     }
//                 }
//                 println!("WebSocket connection closed");
//             }
//         });
//         Ok(Self { tx })
//     }

//     // --- 16 bytes timestamp --- 2 byte path length --- path --- 2 bytes data type --- data ---
//     fn log<T: LoggableData>(&self, path: &str, data: T) -> Result<(), SendError<Command>> {
//         let timestamp = SystemTime::now()
//             .duration_since(UNIX_EPOCH)
//             .unwrap_or(Duration::from_secs(0))
//             .as_nanos();
//         let mut vec = Vec::with_capacity(16 + 2 + path.len() + 2 + data.size());
//         vec.extend_from_slice(&timestamp.to_le_bytes());
//         vec.extend_from_slice(&(path.len() as u16).to_le_bytes());
//         vec.extend_from_slice(path.as_bytes());
//         vec.extend_from_slice(&T::TYPE.to_le_bytes());
//         data.extend_from_bytes(&mut vec);
//         self.tx.send(Command::Data(vec))?;
//         Ok(())
//     }
// }

use kivi::{Box3, MonitorTab};

/// A WebSocket echo server
fn main() {
    let monitor = MonitorTab::new(9876, 9876, 9877);
    for _ in 0..10 {
        monitor.log("test", Box3 { size: [1.0, 2.0, 3.0] }).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
