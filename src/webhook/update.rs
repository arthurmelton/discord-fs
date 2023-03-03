use lazy_static::lazy_static;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{get, get_mut, WEBHOOK};

lazy_static! {
    pub static ref EDIT_TIMES: Mutex<Update> = Mutex::new(Update::new());
}

#[derive(Clone)]
pub struct Update {
    times: [SystemTime; 5],
}

impl Update {
    fn new() -> Update {
        Update {
            times: [UNIX_EPOCH, UNIX_EPOCH, UNIX_EPOCH, UNIX_EPOCH, UNIX_EPOCH],
        }
    }

    pub fn update(&mut self) {
        let since = SystemTime::now().duration_since(self.times[0]).unwrap();
        if Duration::new(10, 0) > since {
            sleep(Duration::new(10, 0) - since);
        }
        self.times.rotate_left(1);
        self.times[4] = SystemTime::now();
    }
}

pub fn update_msg(msg_id: u64, content: Vec<u8>) -> Option<u64> {
    let client = reqwest::blocking::Client::new();
    get_mut!(EDIT_TIMES).update();
    client
        .patch(format!("{}/messages/{}", get!(WEBHOOK), msg_id))
        .multipart(
            reqwest::blocking::multipart::Form::new()
                .part(
                    "files[0]",
                    reqwest::blocking::multipart::Part::bytes(content).file_name("discord-fs"),
                )
                .part(
                    "payload_json",
                    reqwest::blocking::multipart::Part::text("{\"attachments\":[]}"),
                ),
        )
        .send()
        .ok()?
        .json::<serde_json::Value>()
        .ok()?
        .get("attachments")?
        .as_array()?
        .first()?
        .get("id")?
        .as_str()?
        .parse::<u64>().ok()
}
