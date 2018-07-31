use std::io;
use std::io::prelude::*;
use std::io::Read;

use common::MyError;

pub fn parse_user_id(buffer: Vec<u8>) -> Result<String, MyError> {
    let mut cursor = io::Cursor::new(buffer);

    let mut user_id_buf = vec![];

    cursor
        .read_to_end(&mut user_id_buf)
        .map_err(|e| MyError::IoError(e))?;

    return String::from_utf8(user_id_buf.to_vec()).map_err(|e| MyError::ParseError(e));
}

pub fn parse_user_id_bulk(buffer: Vec<u8>) -> Result<Vec<String>, MyError> {
    let buffer_size = buffer.len();
    let mut cursor = io::Cursor::new(buffer);

    let mut user_ids = Vec::new();

    while (cursor.position() as usize) < buffer_size - 1 {
        let mut user_id_buf = vec![];
        cursor
            .read_until(b';', &mut user_id_buf)
            .map_err(|e| MyError::IoError(e))?;
        user_ids.push(
            String::from_utf8(user_id_buf[..user_id_buf.len() - 1].to_vec())
                .map_err(|e| MyError::ParseError(e))?,
        );
    }

    return Ok(user_ids);
}
