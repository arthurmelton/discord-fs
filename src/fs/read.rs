use crate::controller::Item;
use crate::fs::access::check_access;
use crate::{get, get_mut, CHANNEL_ID, FILE_SIZE, FS};
use fuser::{ReplyData, Request};
use libc::{EACCES, ENOENT, ESPIPE};
use crate::fs::write::WRITE_UPDATES;
use std::thread;
use std::time::Duration;

pub fn read(
    req: &Request<'_>,
    ino: u64,
    _fh: u64,
    mut offset: i64,
    size: u32,
    _flags: i32,
    _lock_owner: Option<u64>,
    reply: ReplyData,
) {
    while get!(WRITE_UPDATES).get(&ino).is_some() {
        thread::sleep(Duration::from_secs(1));
    }
    match get_mut!(FS).get_mut(&ino) {
        Some(y) => {
            let attr = y.attr();
            if check_access(
                attr.uid,
                attr.gid,
                attr.permissions,
                req.uid(),
                req.gid(),
                0b100,
            ) {
                match y.clone() {
                    Item::File(x) => {
                        if offset < 0 {
                            offset += x.size as i64;
                        }
                        if offset >= 0 && offset <= x.size as i64 {
                            y.update_last_access();
                            let offset = offset as u64;
                            let start = (offset / FILE_SIZE) as usize;
                            let mut end = (((offset + size as u64) / FILE_SIZE) + 1) as usize;
                            if end > x.message.len() {
                                end = x.message.len();
                            }
                            let first_offset = offset % FILE_SIZE;
                            let mut end_offset = (offset + size as u64) % FILE_SIZE;
                            let mut returns = Vec::new();
                            for i in start..end {
                                let bytes = reqwest::blocking::get(format!(
                                    "https://cdn.discordapp.com/attachments/{}/{}/discord-fs",
                                    get!(CHANNEL_ID),
                                    x.message.get(i).unwrap().1
                                ))
                                .unwrap()
                                .bytes()
                                .unwrap();
                                if i == end-1 && end_offset > bytes.len() as u64 {
                                    end_offset = bytes.len() as u64;
                                }
                                if i == start && i == end-1 {
                                    returns.extend(bytes[first_offset as usize..end_offset as usize].to_vec());
                                } else if i == start {
                                    returns.extend(bytes[first_offset as usize..].to_vec());
                                } else if i == end-1 {
                                    returns.extend(bytes[..end_offset as usize].to_vec());
                                } else {
                                    returns.extend(bytes);
                                }
                            }
                            reply.data(&returns);
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
