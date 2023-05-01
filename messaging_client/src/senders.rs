use std::net::TcpStream;
use std::io::Write;
use std::net::ToSocketAddrs;
use handlers::handle_connection;

const SERVER: &str = "limia.cs.williams.edu:8013";

pub fn initialize(username: &str, ip_addr: &str, port: u16) -> Option<TcpStream>{
    let stream = TcpStream::connect(SERVER.to_socket_addrs().unwrap().next().unwrap());

    match stream {
        Ok(mut server) => {
            let message = ["INIT ".as_bytes(), 
                        username.as_bytes(),
                        ";".as_bytes(),
                        ip_addr.as_bytes(),
                        ":".as_bytes(),
                        port.to_string().as_bytes()].concat();
            _ = server.write(&message);
            _ = server.flush();

            handle_connection(&server, "");
            println!("Welcome to Jaelegram");

            // Set up the server and input stream to be non_blocking
            server.set_nonblocking(true);
            Some(server)
        },
        Err(_) => None,
    }
}

pub fn ip_fetch(recipient: &str, mut server: &TcpStream) -> Option<String> {
    let message = ["IP_FETCH ".as_bytes(), recipient.as_bytes()].concat();
    _ = server.write(&message);
    _ = server.flush();
    Some(String::from("Sent"))
}

pub fn send_message(message: String, mut server: &TcpStream) -> Option<String> {
    _ = server.write(message.trim().as_bytes());
    _ = server.flush();
    Some(String::from("Sent"))
}