use {
    std::{
        fmt::{self, Formatter, Display},
        io::{Error, ErrorKind},
        net::{SocketAddr}
    },
    tokio::{
        net::{TcpStream},
        prelude::{*}
    },
    tokio_codec::{Framed, Decoder},
    codec::{Codec, Request, Response},
    db::{self}
};

pub struct Client {
    io: Framed<TcpStream, Codec>,
    id: Id,
    buffer: Option<Response>,
    db: Db
}

pub enum Db {
    Connecting(db::Db),
    Connected(db::ChannelDb)
}

enum Id {
    Connecting(SocketAddr),
    Connected { addr: SocketAddr, channel: String }
}

impl Client {
    pub fn new<I: Into<Db>>(stream: TcpStream, addr: SocketAddr, db: I) -> Self {
        Client {
            io: Codec::default().framed(stream),
            id: Id::Connecting(addr),
            buffer: None,
            db: db.into()
        }
    }

    fn handle_request(&mut self, request: Request) -> Result<Option<Response>, Error> {
        trace!("{}: handling request: '{:?}'", self.id, request);

        if let Request::Connect { channel } = request {
            let (db, id) = match (&self.id, &self.db) {
                (Id::Connecting(addr), Db::Connecting(db)) => {
                    (db.channel_db(&channel).into(), Id::Connected { addr: addr.clone(), channel })
                }
                (Id::Connected { .. }, Db::Connected(_)) => { return Err(Error::new(ErrorKind::InvalidData, "already connected")); }
                _ => { unreachable!("{}: Id and Db are out of sync", self.id); }
            };

            self.db = db;
            self.id = id;
            
            debug!("{}: connected", self.id);
            return Ok(None);
        }

        let db = if let Db::Connected(db) = &self.db {
            db
        } else {
            return Err(Error::new(ErrorKind::InvalidData, "not connected"));
        };

        Ok(match request {
            Request::Connect { .. } => { unreachable!("{}: connect command should already be handled", self.id); }
            Request::Get { user } => { Some(db.get(&user).into()) }
            Request::BulkEdit { users, points } => {
                if points == 0 {
                    info!("{}: BulkEdit with 0 points does nothing", self.id);
                }

                if points < 0 {
                    users.iter().for_each(|user| { db.update_infallible(user, -(points as i64) as u64, u64::saturating_sub); });
                } else if points > 0 {
                    users.iter().for_each(|user| { db.update_infallible(user, points as u64, u64::saturating_add); });
                }

                None
            }
            Request::Add { user, points } => { Some(Ok(db.update_infallible(user, points, u64::saturating_add)).into()) }
            Request::Remove { user, points, force: false } => { Some(db.update_fallible(user, points, u64::checked_sub).ok_or(points).into()) } 
            Request::Remove { user, points, force: true } => { Some(Ok(db.update_infallible(user, points, u64::saturating_sub)).into()) } 
        })
    }

    fn try_start_send(&mut self, response: Response) -> Poll<(), ()> {
        debug_assert!(self.buffer.is_none());

        match self.io.start_send(response) {
            Ok(AsyncSink::Ready) => { Ok(Async::Ready(())) }
            Ok(AsyncSink::NotReady(response)) => {
                self.buffer = Some(response);
                Ok(Async::NotReady)
            }
            Err(e) => {
                error!("{}: failed to send response: '{}'", self.id, e);
                Err(())
            }
        }
    }
}

impl Future for Client {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(response) = self.buffer.take() {
            trace!("{}: trying to send buffered response...", self.id);

            match self.try_start_send(response)? {
                Async::Ready(()) => { trace!("{}: buffered response sent", self.id); }
                Async::NotReady => {
                    warn!("{}: failed to send buffered response", self.id);
                    return Ok(Async::NotReady);
                }
            }
        }

        loop {
            match self.io.poll() {
                Ok(Async::Ready(Some(request))) => {
                    match self.handle_request(request) {
                        Ok(Some(response)) => {
                            trace!("{}: handled request. sending response: '{:?}'", self.id, response);

                            match self.try_start_send(response)? {
                                Async::Ready(()) => { trace!("{}: response sent", self.id)}
                                Async::NotReady => {
                                    warn!("{}: failed to send response", self.id);
                                    return Ok(Async::NotReady);
                                }
                            }
                        }
                        Ok(None) => { trace!("{}: handled request. no response", self.id); }
                        Err(e) => {
                            error!("{}: failed to handle request: '{}'", self.id, e);
                            return Err(());
                        }
                    }
                }
                Ok(Async::Ready(None)) => {
                    debug!("{}: disconnected", self.id);
                    return Ok(Async::Ready(()));
                }
                Ok(Async::NotReady) => { return Ok(Async::NotReady); }
                Err(e) => {
                    error!("{}: failed to decode request: '{}'", self.id, e);
                    return Err(());
                }
            }
        }
    }
}

impl Display for Id {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Id::Connecting(addr) => { write!(fmt, "{}", addr) }
            Id::Connected { addr, channel } => { write!(fmt, "{}@{}", addr, channel) }
        }
    }
}

impl From<db::Db> for Db {
    fn from(db: db::Db) -> Self {
        Db::Connecting(db)
    }
}

impl From<db::ChannelDb> for Db {
    fn from(db: db::ChannelDb) -> Self {
        Db::Connected(db)
    }
}
