#[macro_export]
macro_rules! send {
    ( $x:expr, $time:expr ) => {{
        use reqwest::header::USER_AGENT;
        use $crate::{get_mut, APP_USER_AGENT, EDIT_TIMES};
        let mut x = None;
        while x.is_none() {
            if $time {
                get_mut!(EDIT_TIMES).update();
            }
            x = $x.header(USER_AGENT, APP_USER_AGENT).send().ok()
        }
        x.unwrap()
    }};
}
