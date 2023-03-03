use crate::controller::Item;
use crate::fs::access::check_access;
use crate::fs::create::make;
use crate::webhook::get_attachment::get_attachment;
use crate::webhook::update::update_msg;
use crate::{get, get_mut, CHANNEL_ID, EDIT_TIMES, FILE_SIZE, FS, WEBHOOK};
use fuser::{ReplyWrite, Request};
use libc::{EACCES, ENOENT, ESPIPE};
use std::time::SystemTime;
use crate::webhook::update_controller::update_controller;

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
    match get_mut!(FS).get_mut(&ino) {
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
                            x.attr.last_modification = SystemTime::now();
                            x.size = offset as u64 + data.len() as u64;
                            let offset = offset as u64;
                            let offset_file = offset / FILE_SIZE;
                            let offset_location = offset % FILE_SIZE;
                            let mut new_files = x.message[..offset_file as usize + 1].to_vec();
                            for i in &x.message[offset_file as usize + 1..] {
                                let client = reqwest::blocking::Client::new();
                                get_mut!(EDIT_TIMES).update();
                                client
                                    .delete(format!("{}/messages/{}", get!(WEBHOOK), i))
                                    .send()
                                    .unwrap();
                            }
                            let mut part = reqwest::blocking::get(format!(
                                "https://cdn.discordapp.com/attachments/{}/{}/discord-fs",
                                get!(CHANNEL_ID),
                                get_attachment(*new_files.last().unwrap()).unwrap()
                            ))
                            .unwrap()
                            .bytes()
                            .unwrap()[..offset_location as usize]
                                .to_vec();
                            let mut taken_data = FILE_SIZE as usize - part.len();
                            if taken_data > data.len() {
                                taken_data = data.len();
                            }
                            let mut data_chunks = vec![data[..taken_data].to_vec()];
                            part.extend(data_chunks.first().unwrap());
                            update_msg(*new_files.last().unwrap(), part);
                            while data.len() - taken_data > 0 {
                                if data.len() - taken_data > FILE_SIZE as usize {
                                    data_chunks.push(
                                        data[taken_data..taken_data + FILE_SIZE as usize].to_vec(),
                                    );
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
    update_controller();
}
