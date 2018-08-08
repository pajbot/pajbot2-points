extern crate bincode;
extern crate byteorder;
extern crate bytes;
extern crate ctrlc;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate serde;
extern crate tokio;
extern crate tokio_codec;

use {
    std::{
        fs::{self, File},
        io::{BufReader, Error, BufWriter, ErrorKind},
        sync::{
            mpsc::{self, RecvTimeoutError},
        },
        time::{Duration},
        thread::{self}
    },
    tokio::{
        prelude::{*},
        runtime::{Runtime}
    },
    db::{Db},
    server::{Server}
};

mod atomic;
mod client;
mod codec;
mod db;
mod server;

const DB_SAVE_INTERVAL: Duration = Duration::from_millis(10 * 1000 * 60);
const DB_SAVE_PATH: &str = "db.txt";
const DB_TMP_SAVE_PATH: &str = "points.db.tmp";
const LISTEN_ADDRESS: &str = "127.0.0.1:54321";

fn main() {
    env_logger::Builder::from_default_env()
        .default_format_timestamp_nanos(true)
        .try_init()
        .expect("failed to initialize env_logger");
        
    let db = load_db().expect("failed to load db");

    let (sender, receiver) = mpsc::channel();
    ctrlc::set_handler(move || {
        info!("termination signal received");
        sender.send(()).unwrap();
    }).expect("failed to set termination signal handler");

    let mut runtime = Runtime::new().expect("failed to create tokio runtime");
    runtime.spawn(Server::new(&LISTEN_ADDRESS.parse().expect("failed to parse socket address"), db.clone()).expect("failed to create server"));

    loop {
        let shutdown = match receiver.recv_timeout(DB_SAVE_INTERVAL) {
            Ok(()) => { true }
            Err(RecvTimeoutError::Disconnected) => {
                error!("termination signal sender disconnected");
                true
            }
            Err(RecvTimeoutError::Timeout) => { false }
        };

        if shutdown {
            info!("shutting down...");
                
            match runtime.shutdown_now().wait() {
                Ok(()) => { info!("runtime shut down"); }
                Err(()) => { error!("runtime shutdown error"); }
            }

            if !db.is_last() {
                warn!("there are still handles to the db");
            }

            save_db_loop(&db);
            break;
        }

        save_db_loop(&db);
    }
}

fn load_db() -> Result<Db, Error> {
    trace!("loading db...");
    
    match File::open(DB_SAVE_PATH) {
        Ok(file) => { bincode::deserialize_from(BufReader::new(file)).map_err(|e| Error::new(ErrorKind::Other, e)) }
        Err(ref e) if e.kind() == ErrorKind::NotFound => { Ok(Default::default()) }
        Err(e) => { Err(e) }
    }
}

fn save_db_loop(db: &Db) {
    for attempt in 0.. {
        log!(
            if attempt == 0 { log::Level::Trace } else { log::Level::Debug },
            "trying to save db for {} time...", attempt + 1
        );

        match save_db_atomic(&db) {
            Ok(()) => {
                log!(
                    if attempt == 0 { log::Level::Debug } else { log::Level::Info },
                    "saved db after {} attempts", attempt + 1
                );

                break;
            }
            Err(e) => {
                error!("{} attempt at saving db failed: '{}'", attempt + 1, e);
            }
        }

        thread::sleep(Duration::from_secs(5));
    }
}

fn save_db_atomic(db: &Db) -> Result<(), Error> {
    let mut writer = BufWriter::new(File::create(DB_TMP_SAVE_PATH)?);
    bincode::serialize_into(&mut writer, &db).map_err(|e| Error::new(ErrorKind::Other, e))?;
    writer.into_inner()?.sync_all()?;
    fs::rename(DB_TMP_SAVE_PATH, DB_SAVE_PATH)
}
