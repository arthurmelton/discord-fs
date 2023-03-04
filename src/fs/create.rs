use crate::controller::{Attr, File, Item};
use crate::fs::access::check_access;
use crate::send;
use crate::{get, get_mut, webhook, FS, TTL, WEBHOOK};
use fuser::{ReplyCreate, Request};
use libc::EACCES;
use std::ffi::OsStr;
use std::time::SystemTime;

pub fn create(
    req: &Request<'_>,
    parent: u64,
    name: &OsStr,
    mode: u32,
    _umask: u32,
    _flags: i32,
    reply: ReplyCreate,
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
            Item::File(File {
                size: 0,
                message: vec![make_empty().unwrap()],
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
        reply.created(&TTL, &get!(FS).get(&inode).unwrap().to_FileAttr(), 0, 0, 0);
    } else {
        reply.error(EACCES);
    }
}

pub fn next_inode() -> u64 {
    let fs = get!(FS);
    let mut fs = fs.iter();
    if fs.len() == 1 {
        fs.next().unwrap().1.attr().ino + 1
    } else {
        fs.max_by_key(|x| x.0).unwrap().0 + 1
    }
}

pub fn make_empty() -> Option<(u64, u64)> {
    make(vec![])
}

pub fn make(content: Vec<u8>) -> Option<(u64, u64)> {
    let client = reqwest::blocking::Client::new();
    let json = send!(
        client
            .post(get!(WEBHOOK))
            .multipart(reqwest::blocking::multipart::Form::new().part(
                "files[0]",
                reqwest::blocking::multipart::Part::bytes(content.clone()).file_name("discord-fs"),
            )),
        true
    )
    .json::<serde_json::Value>()
    .ok()?;
    Some((
        json.get("id")?.as_str()?.parse::<u64>().ok()?,
        json.get("attachments")?
            .as_array()?
            .first()?
            .get("id")?
            .as_str()?
            .parse::<u64>()
            .ok()?,
    ))
}
