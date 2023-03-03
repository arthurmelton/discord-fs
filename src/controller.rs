use lazy_static::lazy_static;

use fuser::FileAttr;
use fuser::FileType;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::SystemTime;

lazy_static! {
    pub static ref FS: Mutex<HashMap<u64, Item>> = Mutex::new(HashMap::new());
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Item {
    File(File),
    Directory(Directory),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct File {
    pub size: u64,
    // this is the message id were the contents is in
    pub message: Vec<u64>,

    pub attr: Attr,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Directory {
    // all the inodes of the inner items
    pub files: Vec<u64>,

    pub attr: Attr,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Attr {
    pub ino: u64,
    pub parent: u64,
    pub name: String,
    pub last_access: SystemTime,
    pub last_modification: SystemTime,
    pub last_change: SystemTime,
    pub creation_time: SystemTime,
    pub permissions: u16,
    pub uid: u32,
    pub gid: u32,
}

macro_rules! to_FileAttr {
    ( $self:expr,  $item:expr ) => {{
        FileAttr {
            ino: $item.attr.ino,
            size: match $self {
                Item::File(x) => x.size,
                Item::Directory(_) => 0,
            },
            blocks: match $self {
                Item::File(x) => (x.size / 512) + 1,
                Item::Directory(_) => 0,
            },
            atime: $item.attr.last_access,
            mtime: $item.attr.last_modification,
            ctime: $item.attr.last_change,
            crtime: $item.attr.creation_time,
            kind: match $self {
                Item::File(_) => FileType::RegularFile,
                Item::Directory(_) => FileType::Directory,
            },
            perm: $item.attr.permissions,
            nlink: match $self {
                Item::File(_) => 1,
                Item::Directory(x) => (x.files.len() + 1) as u32,
            },
            uid: $item.attr.uid,
            gid: $item.attr.gid,
            rdev: 0,
            flags: 0,
            blksize: 512,
        }
    }};
}

impl Item {
    pub fn to_file(&self) -> Option<File> {
        match self {
            Item::File(x) => Some((*x).clone()),
            Item::Directory(_) => None,
        }
    }

    #[allow(non_snake_case)]
    pub fn to_FileAttr(&self) -> FileAttr {
        match self {
            Item::File(x) => to_FileAttr!(self, x),
            Item::Directory(x) => to_FileAttr!(self, x),
        }
    }

    pub fn attr(&self) -> Attr {
        match self {
            Item::File(x) => x.attr.clone(),
            Item::Directory(x) => x.attr.clone(),
        }
    }
    
    pub fn update_last_access(&mut self) {
        match self {
            Item::File(ref mut x) => x.attr.last_access = SystemTime::now(),
            Item::Directory(ref mut x) => x.attr.last_access = SystemTime::now(),
        }
    }
    
    pub fn update_last_change(&mut self) {
        match self {
            Item::File(ref mut x) => x.attr.last_change = SystemTime::now(),
            Item::Directory(ref mut x) => x.attr.last_change = SystemTime::now(),
        }
    }
}
