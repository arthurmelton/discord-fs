use clap::{crate_version, Arg, Command};
use fuser::{Filesystem, MountOption, ReplyAttr, ReplyDirectory, ReplyEntry, Request};
use std::ffi::OsStr;
use std::time::Duration;

mod controller;
mod fs;

use controller::FS;

const TTL: Duration = Duration::from_secs(1); // 1 second

pub struct DiscordFS;

impl Filesystem for DiscordFS {
    fn lookup(&mut self, req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        fs::lookup::lookup(req, parent, name, reply);
    }

    fn getattr(&mut self, req: &Request, ino: u64, reply: ReplyAttr) {
        fs::getattr::getattr(req, ino, reply);
    }

    fn readdir(&mut self, req: &Request, ino: u64, fh: u64, offset: i64, reply: ReplyDirectory) {
        fs::readdir::readdir(req, ino, fh, offset, reply);
    }
}

fn main() {
    let matches = Command::new("discord-fs")
        .version(crate_version!())
        .arg(
            Arg::new("discord-webhook")
                .required(true)
                .index(1)
                .help("The discord webhook, this will comunicate with discord to send data"),
        )
        .arg(
            Arg::new("mount-point")
                .required(true)
                .index(2)
                .help("Act as a client, and mount FUSE at given path"),
        )
        .arg(
            Arg::new("message-token")
                .index(3)
                .help("This will tell the mounter where the main controller file is. If you are running this for the first time dont supply anything but if you are running it again then supply what was givven to you last time you ran it."),
        )
        .arg(
            Arg::new("auto-unmount")
                .long("auto-unmount")
                .help("Automatically unmount on process exit"),
        )
        .arg(
            Arg::new("allow-root")
                .long("allow-root")
                .help("Allow root user to access filesystem"),
        )
        .get_matches();
    let mountpoint = matches.value_of("mount-point").unwrap();
    let mut options = vec![
        MountOption::RO,
        MountOption::FSName("discord-fs".to_string()),
    ];
    if matches.is_present("auto-unmount") {
        options.push(MountOption::AutoUnmount);
    }
    if matches.is_present("allow-root") {
        options.push(MountOption::AllowRoot);
    }
    fuser::mount2(DiscordFS, mountpoint, &options).unwrap();
}
