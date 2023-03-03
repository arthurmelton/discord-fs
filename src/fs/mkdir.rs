use crate::controller::{Attr, Directory, Item};
use crate::fs::access::check_access;
use crate::fs::create::next_inode;
use crate::{get, get_mut, webhook, FS, TTL};
use fuser::{ReplyEntry, Request};
use libc::EACCES;
use std::ffi::OsStr;
use std::time::SystemTime;

pub fn mkdir(
    req: &Request<'_>,
    parent: u64,
    name: &OsStr,
    mode: u32,
    _umask: u32,
    reply: ReplyEntry,
) {
    let inode = next_inode();
    let parent_item = get!(FS).get(&parent).unwrap().clone();
    let attr = parent_item.attr();
    if check_access(
        attr.uid,
        attr.gid,
        attr.permissions,
        req.uid(),
        req.gid(),
        0b010,
    ) {
        if let Item::Directory(mut x) = parent_item.clone() {
            x.files.push(inode);
            *get_mut!(FS).get_mut(&parent).unwrap() = Item::Directory(x.clone());
        }
        get_mut!(FS).insert(
            inode,
            Item::Directory(Directory {
                files: vec![],
                attr: Attr {
                    ino: inode,
                    parent: parent_item.attr().ino,
                    name: name.to_str().unwrap().to_string(),
                    last_access: SystemTime::now(),
                    last_modification: SystemTime::now(),
                    last_change: SystemTime::now(),
                    creation_time: SystemTime::now(),
                    permissions: mode as u16,
                    uid: parent_item.attr().uid,
                    gid: parent_item.attr().gid,
                },
            }),
        );
        webhook::update_controller::update_controller();
        reply.entry(&TTL, &get!(FS).get(&inode).unwrap().to_FileAttr(), 0);
    } else {
        reply.error(EACCES);
    }
}
