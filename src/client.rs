use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, RwLock};

use common::*;
use parse::*;
use read::*;
use utils::*;

pub struct Client {
    stream: TcpStream,
    point_channel_map: ChannelPointMap,
}

impl Client {
    pub fn new(mut stream: TcpStream, points: Arc<Mutex<PointMap>>) -> Result<Client, MyError> {
        let (command, body_size) = read_header(&mut stream)?;
        if command != COMMAND_CONNECT {
            return Err(MyError::WrongCommand(WrongCommand::new(
                command,
                COMMAND_CONNECT,
            )));
        }

        let body_buf = read_body(&mut stream, body_size as usize)?;
        let channel_name = String::from_utf8(body_buf).map_err(|e| MyError::ParseError(e))?;

        let mut data = points.lock().unwrap();
        let channel_point_map = data.entry(channel_name)
            .or_insert(Arc::new(RwLock::new(HashMap::new())));

        return Ok(Client {
            stream: stream,
            point_channel_map: channel_point_map.clone(),
        });
    }

    pub fn run(&mut self) {
        println!("Running client {:?}", self.stream);
        loop {
            match self.handle_command() {
                Err(e) => {
                    // Something that went wrong, went wrong.
                    // If we can recover from the error, or tell the client that something went
                    // wrong, we should probably do that.
                    // For now, disconnecting and letting the client reconnect is probably the best
                    // thing
                    println!("An error occured in handle_command: {}", e);
                    break;
                }
                Ok(_) => {}
            }
        }
    }

    // Blocks and reads + handles the next incoming command
    // TODO: We might want a way to stop at the "waiting for header size" stage in case of quitting
    fn handle_command(&mut self) -> Result<(), MyError> {
        let (command, body_size) = read_header(&mut self.stream)?;
        let body = read_body(&mut self.stream, body_size as usize)?;

        match command {
            COMMAND_GET => {
                let response = self.handle_get_points(body.to_vec())?;
                self.respond(response)?;
            }
            COMMAND_EDIT => {
                let response = self.handle_edit_points(body.to_vec())?;
                self.respond(response)?;
            }
            _ => println!("Unknown command {}", command),
        }

        Ok(())
    }

    fn respond(&mut self, response: Vec<u8>) -> Result<(), MyError> {
        self.stream
            .write(&response)
            .map_err(|e| MyError::IoError(e))?;

        Ok(())
    }

    fn handle_get_points(&mut self, buffer: Vec<u8>) -> Result<Vec<u8>, MyError> {
        let user_id = parse_user_id(buffer.to_vec())?;
        match self.get_points(&user_id) {
            Some(points) => Ok(u64_to_buf(points).to_vec()),
            None => Ok(u64_to_buf(0).to_vec()),
        }
    }

    fn handle_edit_points(&mut self, buffer: Vec<u8>) -> Result<Vec<u8>, MyError> {
        let points_buffer = &buffer[0..4];

        let points = buf_to_i32_unsafe(points_buffer);

        let user_id = parse_user_id(buffer[4..].to_vec())?;

        println!(
            "Add {} points for user with id {} in channel XXX",
            points, user_id
        );

        let mut data = self.point_channel_map.write().unwrap();
        let user_points = data.entry(user_id).or_insert(0);

        match points {
            points if points > 0 => {
                *user_points += points as u64;
            }
            points if points < 0 => {
                let points_u64: u64 = points.abs() as u64;
                if points_u64 > *user_points {
                    *user_points = 0;
                } else {
                    *user_points -= points_u64;
                }
            }
            _ => {
                // Trying to give/remove 0 points, do nothing
            }
        }

        let response = u64_to_buf(*user_points);

        Ok(response.to_vec())
    }

    fn get_points(&mut self, user_id: &String) -> Option<u64> {
        let data = self.point_channel_map.read().unwrap();
        let ret = data.get(user_id);

        match ret {
            Some(x) => {
                return Some(*x);
            }
            None => {
                return None;
            }
        }
    }
}
