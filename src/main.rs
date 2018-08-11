use std::net::TcpListener;

use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::{process, thread, time};

extern crate chrono;

mod common;

mod parse;
mod read;
mod utils;

mod client;
use self::client::Client;
use self::client::Command;

mod points;
use self::points::Points;

extern crate serde;

#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

extern crate bincode;

extern crate ctrlc;

static SAVE_INTERVAL: time::Duration = time::Duration::from_millis(10 * 1000 * 60);
static DB_PATH: &'static str = "db";
static HOST: &'static str = "127.0.0.1:54321";

pub type ChannelPointMap = HashMap<String, u64>;
pub type PointMap = HashMap<String, ChannelPointMap>;

fn main() {
    let points = match Points::load(DB_PATH) {
        Err(e) => {
            println!("Error loading database: {}", e);
            return;
        }
        Ok(p) => p,
    };

    let listener = TcpListener::bind(HOST).unwrap();

    let (sender, receiver) = channel();

    let ctrl_sender_copy = sender.clone();

    // Points handler will have two data structures:
    // 1. Hash map, with a user ID as key, pointing at the users points & rank
    // 2. A sorted list by points, containing the points

    // Initialize points map handler
    thread::spawn(move || {
        loop {
            use Command::*;

            match receiver.recv() {
                Err(_) => continue,
                Ok(cmd) => {
                    println!("Before cmd match");
                    let forward = match &cmd {
                        GetPoints(ref c) => Some(c.channel_name.clone()),
                        BulkEdit(ref c) => Some(c.channel_name.clone()),
                        Edit(ref c) => Some(c.channel_name.clone()),
                        Rank(ref c) => Some(c.channel_name.clone()),
                        SavePoints => {
                            /*
                        let utc: DateTime<Utc> = Utc::now();
                        save_points(&mut points).unwrap();
                        let utc2 = Utc::now();
                        println!("saving took {}", utc2 - utc);
                        */
                            println!("unimplemented");
                            None
                        }
                        Quit(_sender) => {
                            /*
                        let utc: DateTime<Utc> = Utc::now();
                        save_points(&mut points).unwrap();
                        let utc2 = Utc::now();
                        println!("saving took {}", utc2 - utc);
                        sender.send(()).unwrap();
                        */
                            println!("unimplemented");
                            None
                        }
                    };
                    println!("After cmd match");

                    match forward {
                        None => {}
                        Some(channel_name) => {
                            println!("Forwarding command {:?}", cmd);
                            points.forward(channel_name, cmd);
                        }
                    }
                }
            }
        }
    });

    // Initialize SIGINT and SIGTERM handler
    ctrlc::set_handler(move || {
        let (sender, receiver) = channel();
        ctrl_sender_copy.send(Command::Quit(sender)).unwrap();

        receiver.recv().unwrap();

        process::exit(0x0);
    }).expect("Error setting Ctrl-C handler");

    // Initialize occasional sender thread
    let sender_copy = sender.clone();
    thread::spawn(move || loop {
        thread::sleep(SAVE_INTERVAL);
        sender_copy.send(Command::SavePoints).unwrap();
    });

    // Start listening for connections
    for stream_result in listener.incoming() {
        match stream_result {
            Err(e) => println!("Error accepting connection: {}", e),
            Ok(mut stream) => {
                let sender_copy = sender.clone();
                thread::spawn(move || {
                    let result = Client::new(stream, sender_copy);
                    match result {
                        Err(e) => {
                            println!("Error connecting to client: {}", e);
                        }
                        Ok(mut client) => {
                            client.run();
                        }
                    }
                });
            }
        }
    }
}
