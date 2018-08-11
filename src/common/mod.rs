mod custom_error;

pub use self::custom_error::MyError;
pub use self::custom_error::WrongCommand;

pub const COMMAND_CONNECT: u8 = 0x01;
pub const COMMAND_GET: u8 = 0x02;
pub const COMMAND_BULK_EDIT: u8 = 0x03;

pub const COMMAND_ADD: u8 = 0x04;
// Try to remove points. If the user does not have enough points, return an error
pub const COMMAND_REMOVE: u8 = 0x05;

pub const COMMAND_RANK: u8 = 0x06;

pub const RESULT_OK: u8 = 0x00;
pub const RESULT_ERR: u8 = 0x01;
