use crate::{get, WEBHOOK};

pub fn get_attachment(msg_id: u64) -> Option<u64> {
    let res = reqwest::blocking::get(format!("{}/messages/{}", get!(WEBHOOK), msg_id)).ok()?
        .json::<serde_json::Value>().ok()?;
    res.get("attachments")?
        .as_array()?
        .first()?
        .get("id")?
        .as_str()?
        .parse::<u64>().ok()
}
