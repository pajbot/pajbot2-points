use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use std::collections::HashMap;
use std::thread;

mod common;
use self::common::*;

mod parse;
mod read;
mod utils;

mod client;
use self::client::Client;

fn main() {
    let points: Arc<Mutex<PointMap>> = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:54321").unwrap();

    let mut handles = Vec::new();

    for stream_result in listener.incoming() {
        match stream_result {
            Err(e) => println!("Error: {}", e),
            Ok(mut stream) => {
                let points_copy = points.clone();
                let handle = thread::spawn(move || {
                    let result = Client::new(stream, points_copy);
                    match result {
                        Err(e) => {
                            println!("Error connecting to client: {}", e);
                        }
                        Ok(mut client) => {
                            client.run();
                        }
                    }
                    println!("disconnected");
                });

                handles.push(handle);
            }
        }
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
