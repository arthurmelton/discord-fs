use crate::controller::Item;
use crate::fs::access::check_access;
use crate::fs::create::make;
use crate::send;
use crate::webhook::update::update_msg;
use crate::webhook::update_controller::update_controller;
use crate::{get, get_mut, CHANNEL_ID, FILE_SIZE, FS, WEBHOOK};
use fuser::{ReplyWrite, Request};
use lazy_static::lazy_static;
use libc::{EACCES, ENOENT, ESPIPE};
use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime};

lazy_static! {
    pub static ref WRITE_UPDATES: Mutex<HashMap<u64, Vec<Update>>> = Mutex::new(HashMap::new());
}

#[derive(Clone, Debug)]
pub struct Update {
    offset: i64,
    data: Vec<u8>,
}

pub fn write(
    req: &Request<'_>,
    ino: u64,
    _fh: u64,
    mut offset: i64,
    data: &[u8],
    _write_flags: u32,
    _flags: i32,
    _lock_owner: Option<u64>,
    reply: ReplyWrite,
) {
    let mut fs = get_mut!(FS);
    match fs.get_mut(&ino) {
        Some(y) => {
            let attr = y.attr();
            if check_access(
                attr.uid,
                attr.gid,
                attr.permissions,
                req.uid(),
                req.gid(),
                0b010,
            ) {
                match y {
                    Item::File(x) => {
                        if offset < 0 {
                            offset += x.size as i64;
                        }
                        if offset >= 0 && offset <= x.size as i64 {
                            x.size = offset as u64 + data.len() as u64;
                            drop(fs);
                            while match get!(WRITE_UPDATES).get(&ino) {
                                Some(x) => {
                                    x.iter().map(|i| i.data.len()).sum::<usize>() as u64 > FILE_SIZE
                                }
                                None => false,
                            } {
                                thread::sleep(Duration::from_secs(1));
                            }
                            let data = data.to_vec();
                            let mut write_updates = get_mut!(WRITE_UPDATES);
                            match write_updates.get_mut(&ino) {
                                Some(x) => x.push(Update {
                                    offset,
                                    data: data.clone(),
                                }),
                                None => {
                                    write_updates.insert(
                                        ino,
                                        vec![Update {
                                            offset,
                                            data: data.clone(),
                                        }],
                                    );
                                }
                            };
                            reply.written(data.len() as u32);
                        } else {
                            reply.error(ESPIPE)
                        }
                    }
                    Item::Directory(_) => reply.error(ENOENT),
                }
            } else {
                reply.error(EACCES);
            }
        }
        None => reply.error(ENOENT),
    }
}

pub fn write_files() {
    for (ino, mut updates) in get!(WRITE_UPDATES) {
        if let Some(Item::File(x)) = get_mut!(FS).get_mut(&ino) {
            if is_seq(updates.clone()) {
                let offset = updates.first().unwrap().offset;
                let mut data = vec![];
                for i in updates {
                    data.extend(i.data.clone());
                }
                updates = vec![Update { offset, data }];
            }
            for update in updates {
                let offset = update.offset;
                let data = update.data;
                x.attr.last_modification = SystemTime::now();
                let offset = offset as u64;
                let offset_file = offset / FILE_SIZE;
                let offset_location = offset % FILE_SIZE;
                let mut new_files = x.message[..offset_file as usize + 1].to_vec();
                for i in &x.message[offset_file as usize + 1..] {
                    let client = reqwest::blocking::Client::new();
                    send!(
                        client.delete(format!("{}/messages/{}", get!(WEBHOOK), i.0)),
                        true
                    );
                }
                let client = reqwest::blocking::Client::new();
                let mut part = send!(
                    client.get(format!(
                        "https://cdn.discordapp.com/attachments/{}/{}/discord-fs",
                        get!(CHANNEL_ID),
                        new_files.last().unwrap().1
                    )),
                    false
                )
                .bytes()
                .unwrap()[..offset_location as usize]
                    .to_vec();
                let mut taken_data = FILE_SIZE as usize - part.len();
                if taken_data > data.len() {
                    taken_data = data.len();
                }
                let mut data_chunks = vec![data[..taken_data].to_vec()];
                part.extend(data_chunks.first().unwrap());
                let length = new_files.len();
                new_files[length - 1].1 = update_msg(new_files.last().unwrap().0, part).unwrap();
                while data.len() - taken_data > 0 {
                    if data.len() - taken_data > FILE_SIZE as usize {
                        data_chunks
                            .push(data[taken_data..taken_data + FILE_SIZE as usize].to_vec());
                        taken_data += FILE_SIZE as usize;
                    } else {
                        data_chunks.push(data[taken_data..].to_vec());
                        taken_data = data.len();
                    }
                }
                for i in &data_chunks[1..] {
                    new_files.push(make((*i).clone()).unwrap());
                }
                x.message = new_files;
            }
        }
        update_controller();
    }
    *get_mut!(WRITE_UPDATES) = HashMap::new();
}

pub fn is_seq(x: Vec<Update>) -> bool {
    for c in x.windows(2) {
        if c[0].offset + c[0].data.len() as i64 != c[1].offset {
            return false;
        }
    }
    x.len() > 1
}
