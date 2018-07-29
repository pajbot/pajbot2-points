use std::net::{TcpListener, TcpStream};

use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
// use std::io::Read;

fn buf_to_u32(buffer: [u8; 4]) -> u32 {
    return ((buffer[0] as u32) << 24)
        + ((buffer[1] as u32) << 16)
        + ((buffer[2] as u32) << 8)
        + ((buffer[3] as u32));
}

/*
fn buf_to_u32_unsafe(buffer: &[u8]) -> u32 {
    return ((buffer[0] as u32) << 24)
        + ((buffer[1] as u32) << 16)
        + ((buffer[2] as u32) << 8)
        + ((buffer[3] as u32));
}
*/

fn buf_to_i32_unsafe(buffer: &[u8]) -> i32 {
    return ((buffer[0] as i32) << 24)
        + ((buffer[1] as i32) << 16)
        + ((buffer[2] as i32) << 8)
        + ((buffer[3] as i32));
}

fn u64_to_buf(value: u64) -> [u8; 8] {
    let mut buffer = [0; 8];
    buffer[0] = ((value >> 56) & 0xFF) as u8;
    buffer[1] = ((value >> 48) & 0xFF) as u8;
    buffer[2] = ((value >> 40) & 0xFF) as u8;
    buffer[3] = ((value >> 32) & 0xFF) as u8;
    buffer[4] = ((value >> 24) & 0xFF) as u8;
    buffer[5] = ((value >> 16) & 0xFF) as u8;
    buffer[6] = ((value >> 8) & 0xFF) as u8;
    buffer[7] = ((value) & 0xFF) as u8;

    return buffer;
}

enum MyError {
    IoError(io::Error),
    ParseError(std::string::FromUtf8Error),
}

const COMMAND_GET: u8 = 0x01;
const COMMAND_EDIT: u8 = 0x02;

fn parse_channel_user(buffer: &[u8]) -> Result<(String, String), MyError> {
    let mut cursor = io::Cursor::new(buffer);

    let mut channel_name_buf = vec![];
    let mut user_id_buf = vec![];

    cursor
        .read_until(b':', &mut channel_name_buf)
        .map_err(|e| MyError::IoError(e))?;
    cursor
        .read_until(b':', &mut user_id_buf)
        .map_err(|e| MyError::IoError(e))?;

    let channel_name = String::from_utf8(channel_name_buf[0..channel_name_buf.len() - 1].to_vec())
        .map_err(|e| MyError::ParseError(e))?;

    let user_id = String::from_utf8(user_id_buf.to_vec()).map_err(|e| MyError::ParseError(e))?;

    return Ok((channel_name, user_id));
}

fn handle_get_points(
    client: &mut TcpStream,
    buffer: &[u8],
    points: &mut PointMap,
) -> io::Result<()> {
    match parse_channel_user(&buffer) {
        Err(_) => println!("error xd"),
        Ok((channel_name, user_id)) => {
            println!(
                "Get points for user with id {} in channel {}",
                user_id, channel_name
            );
            let channel_point_map = points.entry(channel_name).or_insert(HashMap::new());
            let user_points = channel_point_map.entry(user_id).or_insert(0);
            // Parse channel name and user ID from buffer
            let response_buffer = u64_to_buf(*user_points);

            match client.write(&response_buffer) {
                _ => {}
            }
        }
    }

    Ok(())
}

fn handle_edit_points(
    client: &mut TcpStream,
    buffer: &[u8],
    points_map: &mut PointMap,
) -> io::Result<()> {
    let points_buffer = &buffer[0..4];

    let points = buf_to_i32_unsafe(points_buffer);

    match parse_channel_user(&buffer[4..]) {
        Err(_) => println!("error xd"),
        Ok((channel_name, user_id)) => {
            println!(
                "Add {} points for user with id {} in channel {}",
                points, user_id, channel_name
            );
            let channel_point_map = points_map.entry(channel_name).or_insert(HashMap::new());
            let user_points = channel_point_map.entry(user_id).or_insert(0);

            if points > 0 {
                *user_points += points as u64;
            } else if points < 0 {
                let points_u64: u64 = points.abs() as u64;
                if points_u64 > *user_points {
                    *user_points = 0;
                } else {
                    *user_points -= points_u64;
                }
            } else {
                // nothing to do
            }

            let response_buffer = u64_to_buf(*user_points);

            match client.write(&response_buffer) {
                _ => {}
            }
        }
    }

    Ok(())
}

//      channelName
type ChannelPointMap = HashMap<String, u64>;
type PointMap = HashMap<String, ChannelPointMap>;

fn handle_client(client: &mut TcpStream, mut points: &mut PointMap) -> io::Result<()> {
    let mut header_buffer = [0; 4];

    loop {
        match client.read_exact(&mut header_buffer) {
            // Retry
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,

            Err(_) => return Ok(()),
            Ok(_) => {}
        }

        println!("header {:?}", header_buffer);

        let body_length = buf_to_u32(header_buffer) as usize;

        let mut body_buffer = vec![0; body_length];

        loop {
            match client.read_exact(&mut body_buffer) {
                // Retry
                Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,

                Err(_) => return Ok(()),
                Ok(_) => {}
            }

            println!("Read buffer {:?}", body_buffer);

            let command_byte = body_buffer[0];

            match command_byte {
                COMMAND_GET => handle_get_points(client, &body_buffer[1..], &mut points)?,
                COMMAND_EDIT => handle_edit_points(client, &body_buffer[1..], &mut points)?,
                _ => println!("Unknown command {}", command_byte),
            }

            break;
        }
    }
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:54321").unwrap();

    let mut points = HashMap::new();

    for stream in listener.incoming() {
        println!("connected to {:?}", stream);
        handle_client(&mut stream?, &mut points)?;
        println!("disconnected");
    }

    Ok(())
}
