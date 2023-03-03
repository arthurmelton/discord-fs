use crate::fs::access::check_access;
use crate::fs::unlink::find_in_parent;
use crate::webhook::update_controller::update_controller;
use crate::{get, get_mut, FS};
use fuser::{ReplyEmpty, Request};
use libc::EACCES;
use std::ffi::OsStr;

pub fn rmdir(req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
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
                get_mut!(FS).remove(&x).unwrap();
                update_controller();
                reply.ok();
            }
            Err(x) => reply.error(x),
        }
    } else {
        reply.error(EACCES);
    }
}
