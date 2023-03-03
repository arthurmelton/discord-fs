use crate::{get, FS};
use fuser::{ReplyEmpty, Request};

pub fn access(req: &Request, inode: u64, mask: i32, reply: ReplyEmpty) {
    match get!(FS).get(&inode) {
        Some(item) => {
            let attr = item.attr();
            if check_access(
                attr.uid,
                attr.gid,
                attr.permissions,
                req.uid(),
                req.gid(),
                mask,
            ) {
                reply.ok();
            } else {
                reply.error(libc::EACCES);
            }
        }
        None => reply.error(libc::ENOENT),
    }
}

pub fn check_access(
    file_uid: u32,
    file_gid: u32,
    file_mode: u16,
    uid: u32,
    gid: u32,
    mut access_mask: i32,
) -> bool {
    // F_OK tests for existence of file
    if access_mask == libc::F_OK {
        return true;
    }
    let file_mode = i32::from(file_mode);

    // root is allowed to read & write anything
    if uid == 0 {
        // root only allowed to exec if one of the X bits is set
        access_mask &= libc::X_OK;
        access_mask -= access_mask & (file_mode >> 6);
        access_mask -= access_mask & (file_mode >> 3);
        access_mask -= access_mask & file_mode;
        return access_mask == 0;
    }

    if uid == file_uid {
        access_mask -= access_mask & (file_mode >> 6);
    } else if gid == file_gid {
        access_mask -= access_mask & (file_mode >> 3);
    } else {
        access_mask -= access_mask & file_mode;
    }

    access_mask == 0
}
