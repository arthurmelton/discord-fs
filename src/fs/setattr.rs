use fuser::{Request, TimeOrNow, ReplyAttr};
use std::time::SystemTime;
use crate::{get_mut, FS, TTL};
use crate::controller::{Item, Attr};
use libc::{EACCES, ENOENT};

macro_rules! update {
    ( $self:expr, $atime:expr, $mtime:expr, $ctime:expr, $crtime:expr, $mode:expr, $uid:expr, $gid:expr ) => {{
        $self.attr = Attr {
            ino: $self.attr.ino,
            parent: $self.attr.parent,
            name: $self.attr.name.clone(),
            last_access: to_system_time($atime.unwrap_or(to_TimeOrNow($self.attr.last_access))),
            last_modification: to_system_time($mtime.unwrap_or(to_TimeOrNow($self.attr.last_modification))),
            last_change: $ctime.unwrap_or($self.attr.last_change),
            creation_time: $crtime.unwrap_or($self.attr.creation_time),
            permissions: $mode.unwrap_or($self.attr.permissions as u32) as u16,
            uid: $uid.unwrap_or($self.attr.uid),
            gid: $gid.unwrap_or($self.attr.gid),
        };
    }};
}

pub fn setattr(req: &Request<'_>, ino: u64, mode: Option<u32>, uid: Option<u32>, gid: Option<u32>, _size: Option<u64>, atime: Option<TimeOrNow>, mtime: Option<TimeOrNow>, ctime: Option<SystemTime>, _fh: Option<u64>, crtime: Option<SystemTime>, _chgtime: Option<SystemTime>, _bkuptime: Option<SystemTime>, _flags: Option<u32>, reply: ReplyAttr) {
    match get_mut!(FS).get_mut(&ino) {
        Some(x) => {
            if vec![0, req.uid()].contains(&x.attr().uid) {
                x.update_last_change();
                match x {
                    Item::File(x) => update!(x, atime, mtime, ctime, crtime, mode, uid, gid),
                    Item::Directory(x) => update!(x, atime, mtime, ctime, crtime, mode, uid, gid),
                }
                reply.attr(&TTL, &x.to_FileAttr());
            }
            else {
                reply.error(EACCES);
            }
        }
        None => reply.error(ENOENT),
    }
}

pub fn to_system_time(item: TimeOrNow) -> SystemTime {
    match item {
        TimeOrNow::SpecificTime(x) => x,
        TimeOrNow::Now => SystemTime::now()
    }
}

#[allow(non_snake_case)]
fn to_TimeOrNow(x: SystemTime) -> TimeOrNow {
    TimeOrNow::SpecificTime(x)
}
