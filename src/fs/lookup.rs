use crate::controller::Item;
use crate::{get, FS, TTL};
use fuser::{ReplyEntry, Request};
use libc::{ENOENT, ENOTDIR};
use std::ffi::OsStr;

pub fn lookup(_req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
    let fs = get!(FS);
    let item = fs.get(&parent);
    match item {
        Some(item) => match item {
            Item::File(_) => reply.error(ENOTDIR),
            Item::Directory(item) => {
                for i in item.files.clone().into_iter() {
                    match fs.get(&i) {
                        Some(inner) => {
                            if inner.attr().name.as_str() == name.to_str().unwrap_or("") {
                                reply.entry(&TTL, &inner.to_FileAttr(), 0);
                                return;
                            }
                        }
                        None => {}
                    };
                }
                reply.error(ENOENT);
            }
        },
        None => reply.error(ENOENT),
    }
}
