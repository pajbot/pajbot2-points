use {
    std::{
        io::{Error, ErrorKind},
        str::{self}
    },
    byteorder::{ByteOrder, BE},
    bytes::{Bytes, BytesMut, BufMut},
    tokio_codec::{Decoder, Encoder}
};

#[derive(Default)]
pub struct Codec {
    header: Option<Header>
}

#[derive(Clone, Copy, Debug)]
struct Header {
    command: u8,
    body_len: usize
}

#[derive(Debug)]
pub enum Request {
    Connect { channel: String },
    Get { user: String },
    BulkEdit { users: Vec<String>, points: i32 },
    Add { user: String, points: u64 },
    Remove { user: String, points: u64, force: bool }
}

#[derive(Debug)]
pub enum Response {
    U64(u64),
    ResultU64((bool, u64))
}

impl Decoder for Codec {
    type Item = Request;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let header = match self.header.take() {
            Some(header) => { header }
            None => {
                if src.len() < Header::LEN {
                    return Ok(None);
                }

                src.split_to(Header::LEN).as_ref().into()
            }
        };

        if src.len() >= header.body_len {
            Ok(Some(Request::try_from(header, src.split_to(header.body_len).freeze())?))
        } else {
            self.header = Some(header);
            Ok(None)
        }
    }
}

impl Encoder for Codec {
    type Item = Response;
    type Error = Error;

    fn encode(&mut self, response: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match response {
            Response::U64(v) => {
                dst.reserve(8);
                dst.put_u64_be(v);
            }
            Response::ResultU64((b, v)) => {
                dst.reserve(9);
                dst.put_u8(if b { 0 } else { 1 });
                dst.put_u64_be(v);
            }
        }
        
        Ok(())
    }
}

impl Header {
    const LEN: usize = 5;
}

impl<'a> From<&'a [u8]> for Header {
    fn from(s: &[u8]) -> Self {
        Header {
            command: s[0],
            body_len: BE::read_u32(&s[1..]) as _
        }
    }
}

impl Request {
    fn try_from(header: Header, body: Bytes) -> Result<Self, Error> {
        Ok(match header.command {
            1 => { 
                check_body_len(&body, 1)?;
                Request::Connect { channel: read_string(&body)? }
            }
            2 => {
                check_body_len(&body, 1)?;
                Request::Get { user: read_string(&body)? }
            }
            3 => {
                check_body_len(&body, 5)?;

                Request::BulkEdit {
                    points: BE::read_i32(&body),
                    users: read_str(&body[4..])?.split(';').map(|s| s.to_owned()).collect()
                }
            }
            4 => {
                check_body_len(&body, 9)?;

                Request::Add {
                    points: BE::read_u64(&body),
                    user: read_string(&body[8..])?
                }
            }
            5 => {
                check_body_len(&body, 10)?;

                Request::Remove {
                    force: body[0] != 0,
                    points: BE::read_u64(&body[1..]),
                    user: read_string(&body[9..])?
                }
            }
            _ => { return Err(Error::new(ErrorKind::InvalidData, "invalid command")); }
        })
    }
}

impl From<u64> for Response {
    fn from(v: u64) -> Self {
        Response::U64(v)
    }
}

impl From<Result<u64, u64>> for Response {
    fn from(r: Result<u64, u64>) -> Self {
        Response::ResultU64(match r {
            Ok(v) => { (true, v) }
            Err(v) => { (false, v) }
        })
    }
}

fn check_body_len(body: &[u8], min_len: usize) -> Result<(), Error> {
    if body.len() < min_len {
        Err(Error::new(ErrorKind::InvalidData, "body is too short for command"))
    } else {
        Ok(())
    }
}

fn read_str(bytes: &[u8]) -> Result<&str, Error> {
    debug_assert!(!bytes.is_empty());

    str::from_utf8(bytes).map_err(|_| Error::new(ErrorKind::InvalidData, "non-utf8 string"))
}

fn read_string(bytes: &[u8]) -> Result<String, Error> {
    read_str(bytes).map(|s| s.to_owned())
}
