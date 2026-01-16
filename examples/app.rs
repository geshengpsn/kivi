use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use tungstenite::{Message, accept};

/// A WebSocket echo server
fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9876);
    let server = TcpListener::bind(addr).unwrap();
    loop {
        let (stream, addr) = loop {
            if let Ok(a) = server.accept() {
                break a;
            }
        };
        println!("WebSocket connection accepted from: {}", addr);
        let mut websocket = accept(stream).unwrap();
        loop {
            // websocket.write(Message::Text("Hello, world!".into())).unwrap();
            match websocket.send(Message::Text("Hello, world!".into())) {
                Ok(_) => println!("Message sent"),
                Err(e) => {
                    println!("Error sending message: {}", e);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
