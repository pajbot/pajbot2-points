use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

use common::*;
use parse::*;
use read::*;
use utils::*;

pub struct GetPoints {
    pub channel_name: String,
    pub user_id: String,
    pub response_sender: Sender<u64>,
}

pub struct BulkEdit {
    pub channel_name: String,

    pub user_ids: Vec<String>,

    // How many points to edit (positive for add, negative for remove)
    pub points: i32,
}

pub enum Operation {
    Add,
    Remove,
}

pub struct Edit {
    pub channel_name: String,
    pub user_id: String,

    pub operation: Operation,

    pub value: u64,

    // Force set
    pub force: bool,

    // New value total for user
    pub response_sender: Sender<(bool, u64)>,
}

pub enum Command {
    GetPoints(GetPoints),
    SavePoints,
    Quit(Sender<()>),
    BulkEdit(BulkEdit),
    Edit(Edit),
}

pub struct Client {
    stream: TcpStream,
    // point_channel_map: ChannelPointMap,
    channel_name: String,
    request_sender: Sender<(Command)>,
}

impl Client {
    pub fn new(mut stream: TcpStream, sender: Sender<(Command)>) -> Result<Client, MyError> {
        let (command, body_size) = read_header(&mut stream)?;
        if command != COMMAND_CONNECT {
            return Err(MyError::WrongCommand(WrongCommand::new(
                command,
                COMMAND_CONNECT,
            )));
        }

        let body_buf = read_body(&mut stream, body_size as usize)?;
        let channel_name = String::from_utf8(body_buf).map_err(|e| MyError::ParseError(e))?;

        return Ok(Client {
            stream: stream,
            channel_name: channel_name,
            request_sender: sender,
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
            COMMAND_BULK_EDIT => {
                self.handle_bulk_edit(body.to_vec())?;
            }
            COMMAND_ADD => {
                let response = self.handle_add(body.to_vec())?;
                self.respond(response)?;
            }
            COMMAND_REMOVE => {
                let response = self.handle_remove(body.to_vec())?;
                self.respond(response)?;
            }
            _ => println!("Unknown command {}", command),
        }

        return Ok(());
    }

    fn respond(&mut self, response: Vec<u8>) -> Result<(), MyError> {
        self.stream
            .write(&response)
            .map_err(|e| MyError::IoError(e))?;

        return Ok(());
    }

    fn handle_get_points(&mut self, buffer: Vec<u8>) -> Result<Vec<u8>, MyError> {
        let user_id = parse_user_id(buffer.to_vec())?;

        let (sender, receiver) = channel();

        self.request_sender
            .send(Command::GetPoints(GetPoints {
                channel_name: self.channel_name.clone(),
                user_id: user_id,
                response_sender: sender,
            }))
            .unwrap();

        let points: u64 = receiver.recv().unwrap();

        return Ok(u64_to_buf(points).to_vec());
    }

    fn handle_bulk_edit(&mut self, buffer: Vec<u8>) -> Result<(), MyError> {
        // Read points from 4 first bytes
        let points = buf_to_i32_unsafe(&buffer[0..4]);

        // Read user ID into a string from remaining bytes
        let user_ids = parse_user_id_bulk(buffer[4..].to_vec())?;

        self.request_sender
            .send(Command::BulkEdit(BulkEdit {
                channel_name: self.channel_name.clone(),
                user_ids: user_ids,
                points: points,
            }))
            .unwrap();

        return Ok(());
    }

    fn handle_add(&mut self, buffer: Vec<u8>) -> Result<Vec<u8>, MyError> {
        // Read points from 8 first bytes
        let points = buf_to_u64(&buffer[0..8])?;

        // Read user ID into a string from remaining bytes
        let user_id = parse_user_id(buffer[8..].to_vec())?;

        let (sender, receiver) = channel();

        self.request_sender
            .send(Command::Edit(Edit {
                channel_name: self.channel_name.clone(),
                user_id: user_id,
                operation: Operation::Add,
                value: points,
                force: false,
                response_sender: sender,
            }))
            .unwrap();

        let mut response = Vec::new();

        let (result_bool, user_points) = receiver.recv().unwrap();
        let result = if result_bool { RESULT_OK } else { RESULT_ERR };

        let user_points_buf = u64_to_buf(user_points);

        response.push(result);
        response.append(&mut user_points_buf.to_vec());

        return Ok(response);
    }

    fn handle_remove(&mut self, buffer: Vec<u8>) -> Result<Vec<u8>, MyError> {
        let force = if buffer[0] == 0x01 { true } else { false };
        // Read points from 8 first bytes
        let points = buf_to_u64(&buffer[1..9])?;

        // Read user ID into a string from remaining bytes
        let user_id = parse_user_id(buffer[9..].to_vec())?;

        let (sender, receiver) = channel();

        self.request_sender
            .send(Command::Edit(Edit {
                channel_name: self.channel_name.clone(),
                user_id: user_id,
                operation: Operation::Remove,
                value: points,
                force: force,
                response_sender: sender,
            }))
            .unwrap();

        let mut response = Vec::new();

        let (result_bool, user_points) = receiver.recv().unwrap();
        let result = if result_bool { RESULT_OK } else { RESULT_ERR };

        let user_points_buf = u64_to_buf(user_points);

        response.push(result);
        response.append(&mut user_points_buf.to_vec());

        return Ok(response);
    }
}
