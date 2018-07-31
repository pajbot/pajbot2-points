use std::net::TcpListener;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::{io, process, thread, time};

use std::fs::{File, OpenOptions};
use std::io::prelude::*;

mod common;
use self::common::*;

mod parse;
mod read;
mod utils;

mod client;
use self::client::Client;
use self::client::Command;

use bincode::{deserialize, serialize};

extern crate bincode;

#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

extern crate ctrlc;

static SAVE_INTERVAL: time::Duration = time::Duration::from_millis(10 * 1000 * 60);
static DB_PATH: &'static str = "db.txt";
static HOST: &'static str = "127.0.0.1:54321";

fn load_points() -> io::Result<PointMap> {
    let mut file = File::open(DB_PATH)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    match deserialize(&buf) {
        Err(_) => {
            return Ok(PointMap::new());
        }
        Ok(m) => {
            return Ok(m);
        }
    }
}

fn save_points(points: &mut PointMap) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create(true).open(DB_PATH)?;

    let mut buf = serialize(&points).unwrap();
    file.write(&mut buf)?;

    return Ok(());
}

fn main() {
    let mut points = load_points().unwrap();

    let listener = TcpListener::bind(HOST).unwrap();

    let (sender, receiver) = channel();

    let ctrl_sender_copy = sender.clone();
    let running = Arc::new(AtomicBool::new(true));

    let r = running.clone();

    // Initialize points map handler
    thread::spawn(move || {
        loop {
            use Command::*;

            match receiver.recv() {
                Err(_) => continue,
                Ok(cmd) => match cmd {
                    GetPoints(c) => {
                        let channel_points = points
                            .entry(c.channel_name)
                            .or_insert(ChannelPointMap::new());

                        match channel_points.get(&c.user_id) {
                            Some(x) => {
                                c.response_sender.send(*x).unwrap();
                            }
                            None => {
                                // User did not exist in the points database
                                c.response_sender.send(0).unwrap();
                            }
                        }
                    }
                    EditPoints(c) => {
                        let channel_points = points
                            .entry(c.channel_name)
                            .or_insert(ChannelPointMap::new());

                        let user_points = channel_points.entry(c.user_id).or_insert(0);

                        if c.points > 0 {
                            *user_points += c.points as u64;
                        } else if c.points < 0 {
                            let points_u64: u64 = c.points.abs() as u64;
                            if points_u64 > *user_points {
                                *user_points = 0;
                            } else {
                                *user_points -= points_u64;
                            }
                        } else {
                            // Trying to give/remove 0 points, do nothing
                        }

                        c.response_sender.send(*user_points).unwrap();
                    }
                    SavePoints => {
                        println!("save points");
                        save_points(&mut points).unwrap();
                    }
                    Quit => break,
                },
            }
        }

        save_points(&mut points).unwrap();
        running.store(false, Ordering::SeqCst);
    });

    // Initialize SIGINT and SIGTERM handler
    ctrlc::set_handler(move || {
        ctrl_sender_copy.send(Command::Quit).unwrap();

        while r.load(Ordering::SeqCst) {}

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
