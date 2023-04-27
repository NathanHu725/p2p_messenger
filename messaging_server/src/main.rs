use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use local_ip_address::local_ip;
use mio_pool::poll::{Events, Poll, Token};
use mio::net::TcpStream;
use std::net::TcpListener;
use std::thread;
use threadpool::ThreadPool;

mod handlers;
use handlers::handle_connection;

const PORT: u16 = 8013;

fn setup_server(conn: Arc<Mutex<HashMap<String, TcpStream>>>, 
                cache: Arc<Mutex<HashMap<String, Vec<String>>>>) {
    // Create poll and appropriate objects
    let poll = Arc::new(Mutex::new(Poll::new()))
    let poll_clone = poll.clone();
    let mut events = Events::with_capacity(1024);

        
    thread::spawn(move || {
        let listener = TcpListener::bind(
            format!("{:?}:{}", local_ip().unwrap(), PORT)
        ).unwrap();
    
        // Set up the thread pool
        let counter = 0;
        // let pool = ThreadPool::new(num_workers);
    
        // Each message that comes in is passed to the thread pool
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            poll_clone.lock().unwrap().register(&stream, Token(counter)).unwrap();
            // let temp_conn = conn.clone();
            // let temp_cache = cache.clone();
            // pool.execute(move || {
            //     handle_connection("", stream, temp_conn, temp_cache);
            // });
        }
    });

    let conn = conn.clone();
    let cache = cache.clone();

    loop {
        poll.lock().unwrap().poll(&mut events, None).unwrap();
        for t in &events {
            println!("What is this: {:?}", t);
        }
        // Set up the thread pool
        // let num_workers = 4;
        // let pool = ThreadPool::new(num_workers);

        // for (username, stream) in conn.lock().unwrap().iter() {
        //     let stream = stream.try_clone().expect("failed");
        //     let mut buf = [0; 10];
        //     if let Ok(something) = stream.peek(&mut buf) {
        //         let username = username.clone();
        //         println!("Found 1");
        //         let temp_conn = conn.clone();
        //         let temp_cache = cache.clone();
        //         pool.execute(move || {
        //             handle_connection(&username, stream, temp_conn, temp_cache);
        //         });
        //     }
        // };
    }
}


fn main() {
    let active_connections = Arc::new(Mutex::new(HashMap::new()));
    let cached_messages = Arc::new(Mutex::new(HashMap::new()));
    setup_server(active_connections.clone(), cached_messages.clone());

    println!("Hello, world!");
}