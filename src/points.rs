use chrono::prelude::*;

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use std::{io, thread};

use std::sync::mpsc::{channel, Receiver, Sender};

use client::Command;

use bincode::{deserialize, serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelPoints {
    #[serde(skip_deserializing, skip_serializing)]
    path: String,

    // Key = User ID
    // Value = Rank
    user_id_to_rank: HashMap<String, u64>,

    // Sorted vector of points and User IDs
    ranks: Vec<(u64, String)>,

    #[serde(skip_deserializing, skip_serializing)]
    receiver: Option<Receiver<Command>>,
}

impl ChannelPoints {
    pub fn new(path: &str) -> ChannelPoints {
        return ChannelPoints {
            path: path.to_string(),
            user_id_to_rank: HashMap::new(),
            ranks: Vec::new(),
            receiver: None,
        };
    }

    pub fn load(path: &str) -> io::Result<ChannelPoints> {
        match File::open(path) {
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                return Ok(ChannelPoints::new(path));
            }
            Err(e) => {
                return Err(e);
            }
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;

                match deserialize(&buf) {
                    Err(_) => {
                        return Ok(ChannelPoints::new(path));
                    }
                    Ok(m) => {
                        return Ok(m);
                    }
                }
            }
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.path.clone())?;

        let mut buf = serialize(&self).unwrap();
        file.write(&mut buf)?;

        return Ok(());
    }

    fn edit_points(&mut self, user_id: String, points: i32) -> u64 {
        if points > 0 {
            return self.add_points(user_id, points as u64);
        } else if points < 0 {
            return self.remove_points(user_id, points.abs() as u64);
        }

        return self.get_points(user_id);
    }

    fn add_points(&mut self, user_id: String, points: u64) -> u64 {
        let user_rank = self.user_id_to_rank.entry(user_id).or_insert(0);

        // TODO: Handle rank 0

        let mut res = self.ranks.get_mut(*user_rank as usize);
        match res {
            None => {
                //
            }
            Some((user_points, _)) => {
                *user_points += points;
                // TODO(pajlada): RECALCULATE RANK
                return *user_points;
            }
        }

        return 0;
    }

    fn remove_points(&self, user_id: String, points: u64) -> u64 {
        /*
        let user_points = channel_points.entry(user_id).or_insert(0);

        if points > *user_points {
            *user_points = 0;
        } else {
            *user_points -= points;
        }

        return *user_points;
        */
        return 420;
    }

    fn get_points(&self, user_id: String) -> u64 {
        let user_rank = match self.user_id_to_rank.get(&user_id) {
            None => {
                // User did not exist in the points database
                return 0;
            }
            Some(rank) => rank,
        };

        let user_points = match self.ranks.get(*user_rank as usize) {
            None => {
                return 0;
            }
            Some((points, _)) => points,
        };

        return *user_points;
    }

    pub fn listen(mut self, r: Receiver<Command>) {
        loop {
            use Command::*;
            match r.recv() {
                Err(_) => {
                    // ...
                }
                Ok(cmd) => match cmd {
                    GetPoints(c) => {
                        match self.user_id_to_rank.get(&c.channel_name) {
                            None => {
                                // User did not exist in the points database
                                c.response_sender.send(0).unwrap();
                            }
                            Some(rank) => {
                                // TODO(pajlada): Get points from rank index
                                c.response_sender.send(1337).unwrap();
                            }
                        }
                    }
                    BulkEdit(c) => {
                        for user_id in c.user_ids {
                            self.edit_points(user_id, c.points);
                        }
                    }
                    Edit(c) => {
                        // points.forward(c.channel_name, cmd);
                            /*
                        let channel_points = points
                            .entry(c.channel_name)
                            .or_insert(ChannelPointMap::new());

                        match c.operation {
                            Operation::Add => {
                                let new_value = add_points(channel_points, c.user_id, c.value);
                                c.response_sender.send((true, new_value)).unwrap();
                            }
                            Operation::Remove => {
                                if !c.force {
                                    let user_value = get_points(channel_points, c.user_id.clone());

                                    if user_value < c.value {
                                        c.response_sender.send((false, user_value)).unwrap();
                                        continue;
                                    }
                                }

                                let new_value = remove_points(channel_points, c.user_id, c.value);
                                c.response_sender.send((true, new_value)).unwrap();
                            }
                        }
                        */
                    }
                    Rank(c) => {
                        // points.forward(c.channel_name, cmd);
                            /*
                        let channel_points = points
                            .entry(c.channel_name)
                            .or_insert(ChannelPointMap::new());

                        let user_points = get_points(channel_points, c.user_id.clone());
                        let mut points: Vec<&u64> = Vec::new();
                        for (_other_user_id, other_user_points) in channel_points {
                            points.push(other_user_points);
                        }
                        points.sort();

                        let mut rank = points.len() as u64;
                        for other_user_points in points {
                            if other_user_points < &user_points {
                                rank -= 1;
                            } else {
                                break;
                            }
                        }

                        println!("Responding with rank {}", rank);

                        c.response_sender.send(rank).unwrap();
                        */
                    }
                    SavePoints => {
                        /*
                        let utc: DateTime<Utc> = Utc::now(); // e.g. `2014-11-28T12:45:59.324310806Z`
                        save_points(&mut points).unwrap();
                        let utc2 = Utc::now();
                        println!("saving took {}", utc2 - utc);
                        */
                        println!("unimplemented");
                    }
                    Quit(sender) => {
                        /*
                        let utc: DateTime<Utc> = Utc::now(); // e.g. `2014-11-28T12:45:59.324310806Z`
                        save_points(&mut points).unwrap();
                        let utc2 = Utc::now();
                        println!("saving took {}", utc2 - utc);
                        sender.send(()).unwrap();
                        */
                        println!("unimplemented");
                        break;
                    }
                },
            }
        }
    }
}

#[derive(Debug)]
pub struct Points {
    pub channels: HashMap<String, Sender<Command>>,
}

impl Points {
    fn new() -> Points {
        return Points {
            channels: HashMap::new(),
        };
    }

    fn load_channels(directory: &str) -> io::Result<Points> {
        let mut p = Points::new();

        let db_folder = Path::new(directory);
        for db_file in db_folder.read_dir()? {
            if let Ok(entry) = db_file {
                if let Some(path_str) = entry.path().to_str() {
                    let mut c = ChannelPoints::load(path_str)?;
                    if let Ok(a) = entry.file_name().into_string() {
                        let (sender, receiver) = channel();
                        thread::spawn(move || listen_on_channel(c, receiver));
                        // channels only needs to contain the channel to be able to communicate
                        // with c
                        p.channels.insert(a, sender);
                    }
                }
            }
        }

        return Ok(p);
    }

    pub fn load(path: &str) -> io::Result<Points> {
        let start = Utc::now();
        match Points::load_channels(path) {
            Err(e) => {
                let end = Utc::now();
                println!("Error loading points. Took {}", end - start);
                return Err(e);
            }
            Ok(p) => {
                let end = Utc::now();
                println!("Loading points took {}", end - start);
                return Ok(p);
            }
        }
    }

    pub fn forward(&self, channel_name: String, command: Command) {
        for (channel_name, _) in &self.channels {
            println!("Found channel; {}", channel_name);
        }

        match self.channels.get(&channel_name) {
            None => {
                println!("No sender available");
            }
            Some(sender) => {
                println!("Found a sender in points.channels");
                sender.send(command).unwrap();
                println!("Sent!");
            }
        }
    }
}

fn listen_on_channel(c: ChannelPoints, receiver: Receiver<Command>) {
    c.listen(receiver);
}
