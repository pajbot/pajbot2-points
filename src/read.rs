use std::io;
use std::io::Read;
use std::net::TcpStream;

use common::MyError;
use utils::*;

pub fn read_header(client: &mut TcpStream) -> Result<(u8, u32), MyError> {
    let mut header_buffer = [0; 5];

    loop {
        match client.read_exact(&mut header_buffer) {
            // Retry
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,

            Err(e) => return Err(MyError::IoError(e)),
            Ok(_) => return Ok((header_buffer[0], buf_to_u32_unsafe(&header_buffer[1..]))),
        }
    }
}

pub fn read_body(client: &mut TcpStream, size: usize) -> Result<Vec<u8>, MyError> {
    let mut body_buffer = vec![0; size];

    loop {
        match client.read_exact(&mut body_buffer) {
            // Retry
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,

            Err(e) => return Err(MyError::IoError(e)),
            Ok(_) => return Ok(body_buffer),
        }
    }
}
