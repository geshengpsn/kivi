use std::{
    io::{Read, Write, BufReader, BufRead},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    thread::spawn,
};

use tungstenite::accept;

#[cfg(target_os = "macos")]
pub fn open(url: &str) {
    let _ = std::process::Command::new("open").arg(url).output();
}

#[cfg(target_os = "linux")]
pub fn open(url: &str) {
    let _ = std::process::Command::new("xdg-open").arg(url).output();
}

enum Protocol {
    Http,
    WebSocket,
    RawTcp,
}

fn detect_protocol(stream: &mut TcpStream) -> (Protocol, Vec<u8>) {
    let mut buffer = vec![0u8; 1024];
    let mut total_read = 0;

    // Read initial bytes to detect protocol
    if let Ok(n) = stream.peek(&mut buffer) {
        total_read = n;
        if n > 0 {
            // Check if it starts with HTTP method
            let data = &buffer[..n];
            if let Ok(s) = std::str::from_utf8(data)
                && (s.starts_with("GET ") || s.starts_with("POST ") ||
                   s.starts_with("PUT ") || s.starts_with("DELETE ") ||
                   s.starts_with("HEAD ") || s.starts_with("OPTIONS ")) {
                    // Check if it's a WebSocket upgrade request
                    if s.contains("Upgrade: websocket") || s.contains("upgrade: websocket") {
                        return (Protocol::WebSocket, buffer[..total_read].to_vec());
                    }
                    return (Protocol::Http, buffer[..total_read].to_vec());
                }
        }
    }

    (Protocol::RawTcp, buffer[..total_read].to_vec())
}

fn handle_http(mut stream: TcpStream) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();

    if reader.read_line(&mut request_line).is_ok() {
        println!("HTTP Request: {}", request_line.trim());

        // Read headers
        let mut headers = Vec::new();
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() || line == "\r\n" || line == "\n" {
                break;
            }
            headers.push(line);
        }

        // Send HTTP response
        let response = "HTTP/1.1 200 OK\r\n\
                       Content-Type: text/html; charset=utf-8\r\n\
                       Connection: close\r\n\
                       \r\n\
                       <!DOCTYPE html>\
                       <html>\
                       <head><title>Kivi Monitor</title></head>\
                       <body>\
                       <h1>Kivi Multi-Protocol Server</h1>\
                       <p>This server supports:</p>\
                       <ul>\
                       <li>HTTP (you're viewing this now)</li>\
                       <li>WebSocket (connect to ws://127.0.0.1:9876)</li>\
                       <li>Raw TCP (send binary data directly)</li>\
                       </ul>\
                       </body>\
                       </html>";

        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
    }
}

fn handle_websocket(stream: TcpStream) {
    println!("WebSocket connection established");

    match accept(stream) {
        Ok(mut websocket) => {
            loop {
                match websocket.read() {
                    Ok(msg) => {
                        println!("Received WebSocket message: {:?}", msg);
                        // Echo back the message
                        if websocket.send(msg).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        println!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
            println!("WebSocket connection closed");
        }
        Err(e) => {
            println!("Failed to accept WebSocket: {}", e);
        }
    }
}

fn handle_raw_tcp(mut stream: TcpStream) {
    println!("Raw TCP connection established");

    let mut buffer = [0u8; 4096];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Raw TCP connection closed by client");
                break;
            }
            Ok(n) => {
                println!("Received {} bytes of raw TCP data", n);
                // Echo back the data
                if stream.write_all(&buffer[..n]).is_err() {
                    break;
                }
            }
            Err(e) => {
                println!("Raw TCP error: {}", e);
                break;
            }
        }
    }
}

fn main() {
    let port = 9876;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let server = TcpListener::bind(addr).unwrap();

    println!("Multi-protocol server listening on {}", addr);
    println!("Supports: HTTP, WebSocket, and Raw TCP");

    for stream in server.incoming().flatten() {
        spawn(move || {
            let peer_addr = stream.peer_addr().ok();
            println!("New connection from: {:?}", peer_addr);

            let mut stream_clone = stream.try_clone().unwrap();
            let (protocol, _buffer) = detect_protocol(&mut stream_clone);

            match protocol {
                Protocol::Http => {
                    println!("Detected HTTP protocol");
                    handle_http(stream);
                }
                Protocol::WebSocket => {
                    println!("Detected WebSocket protocol");
                    handle_websocket(stream);
                }
                Protocol::RawTcp => {
                    println!("Detected Raw TCP protocol");
                    handle_raw_tcp(stream);
                }
            }
        });
    }
}
