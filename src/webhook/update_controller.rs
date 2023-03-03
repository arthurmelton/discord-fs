use crate::webhook::update::update_msg;
use crate::{get, FS, MESSAGE_ID};

pub fn update_controller() {
    update_msg(get!(MESSAGE_ID), bincode::serialize(&get!(FS)).unwrap());
}
