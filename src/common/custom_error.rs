use std::fmt;
use std::io;
use std::string;

pub struct WrongCommand {
    received_command: u8,
    expected_command: u8,
}

impl WrongCommand {
    pub fn new(a: u8, b: u8) -> WrongCommand {
        return WrongCommand {
            received_command: a,
            expected_command: b,
        };
    }
}

pub enum MyError {
    IoError(io::Error),
    ParseError(string::FromUtf8Error),
    WrongCommand(WrongCommand),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::IoError(e) => return fmt::Display::fmt(e, f),
            MyError::ParseError(e) => fmt::Display::fmt(e, f),
            MyError::WrongCommand(e) => write!(
                f,
                "wrong command. got {:x}, expected {:x}",
                e.received_command, e.expected_command
            ),
        }
    }
}
