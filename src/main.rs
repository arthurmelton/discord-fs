#![allow(clippy::too_many_arguments)]

use clap::{crate_version, Arg, Command};
use fuser::{
    Filesystem, MountOption, ReplyAttr, ReplyCreate, ReplyData, ReplyDirectory, ReplyEmpty,
    ReplyEntry, ReplyWrite, Request, TimeOrNow,
};
use lazy_static::lazy_static;
use libc::{getegid, geteuid};
use std::ffi::OsStr;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use std::thread;

mod controller;
mod fs;
mod webhook;

use controller::FS;
use webhook::update::EDIT_TIMES;
use fs::write::write_files;

lazy_static! {
    pub static ref USERAGENT: String = format!(
        "discord-fs/{} (https://git.arthurmelton.com/discord-fs)",
        crate_version!()
    );
    pub static ref MESSAGE_ID: Mutex<u64> = Mutex::new(0);
    pub static ref CHANNEL_ID: Mutex<u64> = Mutex::new(0);
    pub static ref WEBHOOK: Mutex<String> = Mutex::new("".to_string());
}
const TTL: Duration = Duration::from_secs(0); // 1 second
const FILE_SIZE: u64 = (7.5 * 1024.0 * 1024.0) as u64;

pub struct DiscordFS;

impl Filesystem for DiscordFS {
    fn lookup(&mut self, req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        fs::lookup::lookup(req, parent, name, reply);
    }

    fn getattr(&mut self, req: &Request, ino: u64, reply: ReplyAttr) {
        fs::getattr::getattr(req, ino, reply);
    }

    fn create(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: ReplyCreate,
    ) {
        fs::create::create(req, parent, name, mode, umask, flags, reply);
    }

    fn readdir(&mut self, req: &Request, ino: u64, fh: u64, offset: i64, reply: ReplyDirectory) {
        fs::readdir::readdir(req, ino, fh, offset, reply);
    }

    fn access(&mut self, req: &Request, inode: u64, mask: i32, reply: ReplyEmpty) {
        fs::access::access(req, inode, mask, reply);
    }

    fn read(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        fs::read::read(req, ino, fh, offset, size, flags, lock_owner, reply);
    }

    fn write(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        fs::write::write(
            req,
            ino,
            fh,
            offset,
            data,
            write_flags,
            flags,
            lock_owner,
            reply,
        );
    }

    fn setattr(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<TimeOrNow>,
        mtime: Option<TimeOrNow>,
        ctime: Option<SystemTime>,
        fh: Option<u64>,
        crtime: Option<SystemTime>,
        chgtime: Option<SystemTime>,
        bkuptime: Option<SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        fs::setattr::setattr(
            req, ino, mode, uid, gid, size, atime, mtime, ctime, fh, crtime, chgtime, bkuptime,
            flags, reply,
        );
    }

    fn mkdir(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        reply: ReplyEntry,
    ) {
        fs::mkdir::mkdir(req, parent, name, mode, umask, reply);
    }

    fn unlink(&mut self, req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        fs::unlink::unlink(req, parent, name, reply);
    }

    fn rmdir(&mut self, req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        fs::rmdir::rmdir(req, parent, name, reply);
    }
}

fn main() {
    env_logger::init();
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
    match webhook::test::test(matches.value_of("discord-webhook").unwrap().to_string()) {
        Ok(x) => {
            *get_mut!(WEBHOOK) = matches.value_of("discord-webhook").unwrap().to_string();
            *get_mut!(CHANNEL_ID) = x;
        }
        Err(x) => match x {
            webhook::test::Error::InvalidURL => error(
                "Invalid webhook url, it should look like https://discord.com/api/webhooks/...",
            ),
            webhook::test::Error::InvalidWebhook => {
                error("Invalid webhook, make sure this web hook actually works")
            }
            webhook::test::Error::InvalidNetwork => {
                error("Your network is not working, or discord is down")
            }
        },
    }
    match matches.value_of("message-token") {
        Some(x) => match x.parse::<u64>() {
            Ok(x) => {
                *get_mut!(MESSAGE_ID) = x;
                let attachment = webhook::get_attachment::get_attachment(get!(MESSAGE_ID));
                if attachment.is_none() {
                    error("The message token you provided did not work ;(");
                }
                *get_mut!(FS) = bincode::deserialize(
                    &reqwest::blocking::get(format!(
                        "https://cdn.discordapp.com/attachments/{}/{}/discord-fs",
                        get!(CHANNEL_ID),
                        attachment.unwrap()
                    ))
                    .unwrap()
                    .bytes()
                    .unwrap(),
                )
                .unwrap();
            }
            Err(_) => error("message-token is not a valid u64"),
        },
        None => {
            get_mut!(FS).insert(
                1,
                controller::Item::Directory(controller::Directory {
                    files: vec![],
                    attr: controller::Attr {
                        ino: 1,
                        parent: 1,
                        name: "".to_string(),
                        last_access: SystemTime::now(),
                        last_modification: SystemTime::now(),
                        last_change: SystemTime::now(),
                        creation_time: SystemTime::now(),
                        permissions: 0o755,
                        uid: unsafe { geteuid() },
                        gid: unsafe { getegid() },
                    },
                }),
            );
            let id = fs::create::make_empty().unwrap();
            *get_mut!(MESSAGE_ID) = id.0;
            println!("Next time you run the program pass this as the message-token: {}", id.0);
            webhook::update_controller::update_controller();
        }
    }
    let mountpoint = matches.value_of("mount-point").unwrap();
    let mut options = vec![
        MountOption::RW,
        MountOption::FSName("discord-fs".to_string()),
    ];
    if matches.is_present("auto-unmount") {
        options.push(MountOption::AutoUnmount);
    }
    if matches.is_present("allow-root") {
        options.push(MountOption::AllowRoot);
    }
    thread::spawn(|| {
        loop {
            write_files();
            thread::sleep(Duration::from_secs(1));
        }
    });
    fuser::mount2(DiscordFS, mountpoint, &options).unwrap();
}

fn error(msg: &str) {
    eprint!("{msg}");
    std::process::exit(1);
}

#[macro_export]
macro_rules! get {
    ( $x:expr ) => {{
        (*$x.lock().unwrap()).clone()
    }};
}

#[macro_export]
macro_rules! get_mut {
    ( $x:expr ) => {{
        #[allow(unused_mut)]
        let mut x = $x.lock().unwrap();
        x
    }};
}
