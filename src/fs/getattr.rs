use crate::fs::access::check_access;
use crate::{get, FS, TTL};
use fuser::{ReplyAttr, Request};
use libc::ENOENT;

pub fn getattr(req: &Request, ino: u64, reply: ReplyAttr) {
    let fs = get!(FS);

    match fs.get(&ino) {
        Some(x) => {
            if ino == 1 {
                reply.attr(&TTL, &x.to_FileAttr())
            } else {
                let parent = fs.get(&x.attr().parent).unwrap().clone();
                let attr = parent.attr();
                if check_access(
                    attr.uid,
                    attr.gid,
                    attr.permissions,
                    req.uid(),
                    req.gid(),
                    0b100,
                ) {
                    reply.attr(&TTL, &x.to_FileAttr())
                } else {
                    reply.error(ENOENT)
                }
            }
        }
        None => reply.error(ENOENT),
    }
}
