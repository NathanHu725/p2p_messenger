use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use local_ip_address::local_ip;
use std::net::{TcpListener, TcpStream};
use std::thread;
use threadpool::ThreadPool;

use handlers::handle_connection;

const PORT: u16 = 8013;

fn setup_server(conn: Arc<Mutex<HashMap<String, TcpStream>>>, 
                cache: Arc<Mutex<HashMap<String, Vec<String>>>>) {
    // thread::spawn(move || {
        let listener = TcpListener::bind(
            format!("{}:{}", local_ip().unwrap(), PORT)
        ).unwrap();
    
        // Set up the thread pool
        let num_workers = 4;
        let pool = ThreadPool::new(num_workers);
    
        // Each message that comes in is passed to the thread pool
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let temp_conn = conn.clone();
            let temp_cache = cache.clone();
            pool.execute(move || {
                handle_connection(stream, temp_conn, temp_cache);
            });
        }
    // });
}


fn main() {
    let mut active_connections = Arc::new(Mutex::new(HashMap::new()));
    let mut cached_messages = Arc::new(Mutex::new(HashMap::new()));
    setup_server(active_connections.clone(), cached_messages.clone());

    println!("Hello, world!");
}