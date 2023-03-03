use crate::{get, FS, MESSAGE_ID};
use crate::webhook::update::update_msg;

pub fn update_controller() {
    update_msg(get!(MESSAGE_ID), bincode::serialize(&get!(FS)).unwrap());
}
