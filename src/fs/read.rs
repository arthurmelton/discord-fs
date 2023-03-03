use crate::controller::Item;
use crate::fs::access::check_access;
use crate::{get, get_mut, CHANNEL_ID, FILE_SIZE, FS};
use fuser::{ReplyData, Request};
use libc::{EACCES, ENOENT, ESPIPE};
use crate::fs::write::WRITE_UPDATES;
use std::thread;
use std::time::Duration;
use reqwest::header::{HeaderMap, RANGE, HeaderValue};
use crate::send;

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
                            let mut end = (((offset + size as u64 - 1) / FILE_SIZE) + 1) as usize;
                            if end > x.message.len() {
                                end = x.message.len();
                            }
                            let first_offset = offset % FILE_SIZE;
                            let end_offset = (offset + size as u64 - 1) % FILE_SIZE;
                            let mut returns = Vec::new();
                            for i in start..end {
                                let mut headers = HeaderMap::new();
                                if i == start && i == end-1 {
                                    headers.insert(RANGE, HeaderValue::from_str(format!("bytes={first_offset}-{end_offset}").as_str()).unwrap());
                                } else if i == start {
                                    headers.insert(RANGE, HeaderValue::from_str(format!("bytes={first_offset}-").as_str()).unwrap());
                                } else if i == end-1 {
                                    headers.insert(RANGE, HeaderValue::from_str(format!("bytes=-{end_offset}").as_str()).unwrap());
                                }
                                let client = reqwest::blocking::Client::new();
                                let bytes = send!(client.get(format!(
                                    "https://cdn.discordapp.com/attachments/{}/{}/discord-fs",
                                    get!(CHANNEL_ID),
                                    x.message.get(i).unwrap().1
                                ))
                                    .headers(headers.clone()), false)
                                    .bytes()
                                    .unwrap();
                                returns.extend(bytes);

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
