use std::collections::HashMap;

mod custom_error;

pub use self::custom_error::MyError;
pub use self::custom_error::WrongCommand;

pub const COMMAND_CONNECT: u8 = 0x01;
pub const COMMAND_GET: u8 = 0x02;
pub const COMMAND_EDIT: u8 = 0x03;
pub const COMMAND_BULK_EDIT: u8 = 0x04;

pub type ChannelPointMap = HashMap<String, u64>;
pub type PointMap = HashMap<String, ChannelPointMap>;
