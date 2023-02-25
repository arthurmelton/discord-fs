use crate::{get, FS, TTL};
use fuser::{ReplyAttr, Request};
use libc::ENOENT;

pub fn getattr(_req: &Request, ino: u64, reply: ReplyAttr) {
    match get!(FS).get(&ino) {
        Some(x) => reply.attr(&TTL, &x.to_FileAttr()),
        None => reply.error(ENOENT),
    }
}
