use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::Deserialize;
use teloxide::types::{
    User,
    UserId
};

#[derive(Debug, PartialEq)]
pub enum Error {
    BadData(serde_urlencoded::de::Error),
    BadArgs,
    TooOld,
    HashMismatch,
    /// Returned if bot api contracts are violated
    WTF,
}

/// Parses and validates [Telegram.WebApp.initData](https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app)
///
/// Errors
/// TooOld if token is older than 30 minutes
pub fn validate(data: &[u8], token: &[u8], ignore_age: bool) -> Result<User, Error> {
    use Error::*;

    if data.len() == 0 || token.len() == 0 { return Err(BadArgs) }

    // parse init_data into fields
    let mut pairs: HashMap<String, String> =
        match serde_urlencoded::from_bytes(&data) {
            Ok(pairs) => pairs,
            Err(e) => return Err(BadData(e))
        };

    // check if hash field is in place
    let hash = match pairs.remove("hash") {
        Some(hash) => hash,
        None => return Err(WTF)
    };

    // check if auth_date is in place and not too old
    if !ignore_age {
        match pairs.get("auth_date") {
            None => { return Err(WTF) }
            Some(seconds) => {
                let seconds: u64 = match seconds.parse() {
                    Ok(seconds) => seconds,
                    Err(_) => return Err(WTF)
                };

                if SystemTime::now() - Duration::from_secs(seconds) > UNIX_EPOCH + Duration::from_secs(1800) {
                    return Err(TooOld);
                }
            }
        }
    }

    // form data_check_string
    let mut keys: Vec<&String> = pairs.keys().collect();
    keys.sort();

    let mut data_check_string = String::with_capacity(300);
    for key in keys {
        data_check_string.push_str(key);
        data_check_string.push_str("=");
        data_check_string.push_str(&pairs.get(key).unwrap());
        data_check_string.push_str("\n");
    }

    // derive a key from bot token
    let mut mac = Hmac::<Sha256>::new_from_slice(b"WebAppData").unwrap();
    mac.update(token);

    // calculate actual hash
    let mut mac = Hmac::<Sha256>::new_from_slice(&mac.finalize().into_bytes()[..]).unwrap();
    mac.update(data_check_string.trim().as_bytes());

    // compare it with received
    if mac.finalize().into_bytes()[..] != hex::decode(hash).unwrap()[..] {
        return Err(HashMismatch);
    }

    // return User
    let u = pairs.remove("user").unwrap();
    Ok(serde_json::from_str::<WebAppUser>(&u).unwrap().into())
}

#[derive(Deserialize, Debug)]
struct WebAppUser {
    id: u64,
    first_name: String,
    last_name: Option<String>,
    username: Option<String>,
    language_code: Option<String>,
    is_premium: Option<bool>,
    allows_write_to_pm: bool,
}

impl From<WebAppUser> for User {
    fn from(value: WebAppUser) -> Self {
        User {
            id: UserId(value.id),
            is_bot: false,
            first_name: value.first_name,
            last_name: value.last_name,
            username: value.username,
            language_code: value.language_code,
            is_premium: value.is_premium.unwrap_or_default(),
            added_to_attachment_menu: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use teloxide::prelude::*;
    use teloxide::types::User;
    use super::Error::TooOld;
    use super::{validate, WebAppUser};

    #[test]
    fn user_parsed_successfully() {
        let u = r#"{"id":113472905,"first_name":"Leonid","last_name":"Burdikov","username":"reina_bailando","language_code":"en","is_premium":true,"allows_write_to_pm":true}"#;

        let x: WebAppUser = serde_json::from_str(u).unwrap();

        assert_eq!(User::from(x), User{
            id: UserId(113472905),
            is_bot: false,
            first_name: "Leonid".to_string(),
            last_name: Some(String::from("Burdikov")),
            username: Some(String::from("reina_bailando")),
            language_code: Some(String::from("en")),
            is_premium: true,
            added_to_attachment_menu: false,
        })
    }

    #[test]
    fn too_old() {
        let init_data = b"query_id=AAGJdcMGAAAAAIl1wwaf8-89&user=%7B%22id%22%3A113472905%2C%22first_name%22%3A%22Leonid%22%2C%22last_name%22%3A%22Burdikov%22%2C%22username%22%3A%22reina_bailando%22%2C%22language_code%22%3A%22en%22%2C%22is_premium%22%3Atrue%2C%22allows_write_to_pm%22%3Atrue%7D&auth_date=1724270665&hash=47f6068f83ce0a2af458c6ee57f33adf7695d636ba51e0668b067d17fd04fdb2";
        let token = b"7214402729:AAEN53HK_2QKc2shfAopG4SybaQu_hpReS0";

        let res = validate(init_data, token, false);
        assert_eq!(res, Err(TooOld))
    }

    #[test]
    fn valid_with_ignored_age() {
        let init_data = b"query_id=AAGJdcMGAAAAAIl1wwaf8-89&user=%7B%22id%22%3A113472905%2C%22first_name%22%3A%22Leonid%22%2C%22last_name%22%3A%22Burdikov%22%2C%22username%22%3A%22reina_bailando%22%2C%22language_code%22%3A%22en%22%2C%22is_premium%22%3Atrue%2C%22allows_write_to_pm%22%3Atrue%7D&auth_date=1724270665&hash=47f6068f83ce0a2af458c6ee57f33adf7695d636ba51e0668b067d17fd04fdb2";
        let token = b"7214402729:AAEN53HK_2QKc2shfAopG4SybaQu_hpReS0";

        let res = validate(init_data, token, true);
        assert!(res.is_ok())
    }
}