use std::net::TcpStream;
use std::io::Write;

const SERVER: &str = "https://limia.cs.williams.edu:8013";

pub fn initialize(username: &str, ip_addr: &str, port: u16) -> Option<TcpStream>{
    let mut stream = TcpStream::connect(SERVER);

    if let Ok(mut server) = stream {
        let message = ["INIT ".as_bytes(), 
                        username.as_bytes(),
                        ";".as_bytes(),
                        ip_addr.as_bytes(),
                        ":".as_bytes(),
                        &port.to_ne_bytes()].concat();
        server.write(&message);
        server.flush();
        Some(server);
    }

    None
}

pub fn ip_fetch(recipient: &str, mut server: &TcpStream) -> Option<String> {
    let message = ["IP_FETCH ".as_bytes(), recipient.as_bytes()].concat();
    server.write(&message);
    server.flush();
    Some(String::from("Sent"))
}

pub fn send_message(message: String, mut server: &TcpStream) -> Option<String> {
    server.write(message.as_bytes());
    server.flush();
    Some(String::from("Sent"))
}