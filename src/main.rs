use clap::{crate_version, Arg, Command};
use fuser::{Filesystem, MountOption, ReplyAttr, ReplyDirectory, ReplyEntry, Request};
use lazy_static::lazy_static;
use libc::{getegid, geteuid};
use std::ffi::OsStr;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;

mod controller;
mod fs;
mod webhook;

use controller::FS;
use webhook::update::EDIT_TIMES;

lazy_static! {
    pub static ref USERAGENT: String = format!(
        "discord-fs/{} (https://git.arthurmelton.com/discord-fs)",
        crate_version!()
    );
    pub static ref MESSAGE_ID: Mutex<u64> = Mutex::new(0);
    pub static ref CHANNEL_ID: Mutex<u64> = Mutex::new(0);
    pub static ref WEBHOOK: Mutex<String> = Mutex::new("".to_string());
}
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
    match webhook::test::test(matches.value_of("discord-webhook").unwrap().to_string()) {
        Ok(x) => {
            let mut webhook = WEBHOOK.lock().unwrap();
            *webhook = matches.value_of("discord-webhook").unwrap().to_string();
            drop(webhook);
            let mut channel = CHANNEL_ID.lock().unwrap();
            *channel = x;
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
                let mut message_id = MESSAGE_ID.lock().unwrap();
                *message_id = x;
                drop(message_id);
                let attachment = webhook::get_attachment::get_attachment(get!(MESSAGE_ID));
                if attachment.is_none() {
                    error("The message token you provided did not work ;(");
                }
                let mut fs = FS.lock().unwrap();
                *fs = bincode::deserialize(
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
            let mut fs = FS.lock().unwrap();
            fs.insert(
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
            drop(fs);
            let client = reqwest::blocking::Client::new();
            get!(EDIT_TIMES).update();
            let res = client
                .post(get!(WEBHOOK))
                .multipart(reqwest::blocking::multipart::Form::new().part(
                    "files[0]",
                    reqwest::blocking::multipart::Part::text("tmp").file_name("discord-fs"),
                ))
                .send()
                .unwrap()
                .json::<serde_json::Value>()
                .unwrap();
            let mut message_id = MESSAGE_ID.lock().unwrap();
            *message_id = res
                .get("id")
                .unwrap()
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap();
            println!(
                "Next time you run the program pass this as the message-token: {message_id}"
            );
            drop(message_id);
            webhook::update_controller::update_controller();
        }
    }
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
