use lazy_static::lazy_static;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
        if Duration::new(5, 0) > since {
            sleep(Duration::new(5, 0) - since);
        }
        self.times[0] = self.times[1];
        self.times[1] = self.times[2];
        self.times[2] = self.times[3];
        self.times[3] = self.times[4];
        self.times[4] = SystemTime::now();
    }
}
