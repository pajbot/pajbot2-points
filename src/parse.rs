use std::io;
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
