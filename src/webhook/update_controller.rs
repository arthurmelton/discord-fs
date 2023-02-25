use crate::webhook::update::EDIT_TIMES;
use crate::{get, FS, MESSAGE_ID, WEBHOOK};

pub fn update_controller() {
    let client = reqwest::blocking::Client::new();
    get!(EDIT_TIMES).update();
    client
        .patch(format!("{}/messages/{}", get!(WEBHOOK), get!(MESSAGE_ID)))
        .multipart(
            reqwest::blocking::multipart::Form::new()
                .part(
                    "files[0]",
                    reqwest::blocking::multipart::Part::bytes(
                        bincode::serialize(&get!(FS)).unwrap(),
                    )
                    .file_name("discord-fs"),
                )
                .part(
                    "payload_json",
                    reqwest::blocking::multipart::Part::text("{\"attachments\":[]}"),
                ),
        )
        .send()
        .unwrap();
}
