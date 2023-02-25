use crate::{FS, TTL};
use fuser::{ReplyAttr, Request};
use libc::ENOENT;

pub fn getattr(_req: &Request, ino: u64, reply: ReplyAttr) {
    let fs = FS.lock().unwrap();
    match fs.get(&ino) {
        Some(x) => reply.attr(&TTL, &x.to_FileAttr()),
        None => reply.error(ENOENT),
    }
}
