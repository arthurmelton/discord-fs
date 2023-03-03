use crate::controller::Item;
use crate::fs::access::check_access;
use crate::webhook::update_controller::update_controller;
use crate::{get, get_mut, FS, WEBHOOK};
use fuser::{ReplyEmpty, Request};
use libc::{EACCES, ENOENT, ENOTDIR};
use std::ffi::{c_int, OsStr};
use crate::send;

pub fn unlink(req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
    let parent_item = get!(FS).get(&parent).unwrap().clone();
    let attr = parent_item.attr();
    if check_access(
        attr.uid,
        attr.gid,
        attr.permissions,
        req.uid(),
        req.gid(),
        0b011,
    ) {
        match find_in_parent(parent, name.to_str().unwrap().to_string()) {
            Ok(x) => {
                for i in get!(FS).get(&x).unwrap().to_file().unwrap().message {
                    let client = reqwest::blocking::Client::new();
                    send!(client
                        .delete(format!("{}/messages/{}", get!(WEBHOOK), i.0)), true
                    );
                }
                get_mut!(FS).remove(&x).unwrap();
                update_controller();
                reply.ok();
            }
            Err(x) => reply.error(x),
        }
        update_controller();
    } else {
        reply.error(EACCES);
    }
}

pub fn find_in_parent(parent: u64, name: String) -> Result<u64, c_int> {
    let fs = get!(FS);
    match fs.get(&parent) {
        Some(x) => match x {
            Item::File(_) => Err(ENOTDIR),
            Item::Directory(x) => {
                let file: Vec<u64> = x
                    .files
                    .iter()
                    .filter_map(|y| fs.get(y))
                    .filter(|y| y.attr().name == name)
                    .map(|y| y.attr().ino)
                    .collect();
                match file.first() {
                    Some(x) => Ok(*x),
                    None => Err(ENOENT),
                }
            }
        },
        None => Err(ENOENT),
    }
}
