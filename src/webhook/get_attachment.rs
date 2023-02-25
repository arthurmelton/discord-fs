use crate::{get, WEBHOOK};

pub fn get_attachment(msg_id: u64) -> u64 {
    let res = reqwest::blocking::get(format!("{}/messages/{}", get!(WEBHOOK), msg_id))
        .unwrap()
        .json::<serde_json::Value>()
        .unwrap();
    res.get("attachments")
        .unwrap()
        .as_array()
        .unwrap()
        .first()
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap()
}
