use crate::controller::Item;
use crate::{get, FS};
use fuser::{FileType, ReplyDirectory, Request};
use libc::ENOENT;

pub fn readdir(_req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
    let fs = get!(FS);

    match fs.get(&ino) {
        Some(x) => match x {
            Item::File(_) => reply.error(ENOENT),
            Item::Directory(x) => {
                let mut entries = vec![
                    (ino, FileType::Directory, ".".to_string()),
                    (x.attr.parent, FileType::Directory, "..".to_string()),
                ];

                for i in x.files.clone().into_iter() {
                    if let Some(x) = fs.get(&i) {
                        entries.push((x.attr().ino, x.to_FileAttr().kind, x.attr().name));
                    }
                }
                for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
                    if reply.add(entry.0, (i + 1) as i64, entry.1, entry.2) {
                        break;
                    }
                }
                reply.ok();
            }
        },
        None => reply.error(ENOENT),
    }
}
