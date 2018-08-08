use {
    std::{
        io::{Result},
        net::{SocketAddr}
    },
    tokio::{
        self,
        net::{TcpListener},
        prelude::{*}
    },
    client::{Client},
    db::{Db}
};

pub struct Server {
    listener: TcpListener,
    db: Db
}

impl Server {
    pub fn new(addr: &SocketAddr, db: Db) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;
        debug!("listening at {}...", addr);

        Ok(Server { listener, db })
    }
}

impl Future for Server {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match self.listener.poll_accept() {
                Ok(Async::Ready((stream, addr))) => {
                    debug!("accepted connection: '{}'", addr);
                    tokio::spawn(Client::new(stream, addr, self.db.clone()));
                }
                Ok(Async::NotReady) => { return Ok(Async::NotReady); }
                Err(e) => { error!("failed to accept connection: '{}'", e); }
            }
        }
    }
}
