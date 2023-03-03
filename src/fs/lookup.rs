use crate::controller::Item;
use crate::{get, FS, TTL};
use fuser::{ReplyEntry, Request};
use libc::{ENOENT, ENOTDIR, EACCES};
use std::ffi::OsStr;
use crate::fs::access::check_access;

pub fn lookup(req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
    let fs = get!(FS);
    let item = fs.get(&parent);
    match item {
        Some(item) => {
            let attr = item.attr();
            if check_access(attr.uid, attr.gid, attr.permissions, req.uid(), req.gid(), 0b100) {
                match item {
                    Item::File(_) => reply.error(ENOTDIR),
                    Item::Directory(item) => {
                        for i in item.files.clone().into_iter() {
                            if let Some(inner) = fs.get(&i) {
                                if inner.attr().name.as_str() == name.to_str().unwrap_or("") {
                                    reply.entry(&TTL, &inner.to_FileAttr(), 0);
                                    return;
                                }
                            }
                        }
                        reply.error(ENOENT);
                    }
                }
            }
            else {
                reply.error(EACCES);
            }
        },
        None => reply.error(ENOENT),
    }
}
