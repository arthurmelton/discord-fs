use regex::Regex;

pub enum Error {
    InvalidURL,
    InvalidWebhook,
    InvalidNetwork,
}

pub fn test(url: String) -> Result<u64, Error> {
    let re = Regex::new(r"^https://[^.]*\.?discord\.com/api/webhooks/[0-9]*/[^/]*$").unwrap();
    if !re.is_match(&url) {
        return Err(Error::InvalidURL);
    }

    match reqwest::blocking::get(url) {
        Ok(request) => match request.json::<serde_json::Value>() {
            Ok(request) => match request.get("channel_id") {
                Some(x) => Ok(x.as_str().unwrap().parse::<u64>().unwrap()),
                None => Err(Error::InvalidWebhook),
            },
            Err(_) => Err(Error::InvalidWebhook),
        },
        Err(_) => Err(Error::InvalidNetwork),
    }
}
