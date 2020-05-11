//! Requests
//!
//! Send requests and decode responses

use serde;
use std::time::Instant;
use floating_duration::TimeFormat;

use crate::error::CError;
use super::super::ClientShim;
use super::super::Result;

pub fn post<V>(client_shim: &ClientShim, path: &str) -> Result<V>
    where V: serde::de::DeserializeOwned
{
    _postb(client_shim, path, "{}")
}

pub fn postb<T, V>(client_shim: &ClientShim, path: &str, body: T) -> Result<V>
where
    T: serde::ser::Serialize,
    V: serde::de::DeserializeOwned
{
    _postb(client_shim, path, body)
}

fn _postb<T, V>(client_shim: &ClientShim, path: &str, body: T) -> Result<V>
    where
        T: serde::ser::Serialize,
        V: serde::de::DeserializeOwned
{
    let start = Instant::now();

    let mut b = client_shim
        .client
        .post(&format!("{}/{}", client_shim.endpoint, path));

    if client_shim.auth_token.is_some() {
        b = b.bearer_auth(client_shim.auth_token.clone().unwrap());
    }

    // catch reqwest errors
    let value = match b.json(&body).send() {
        Ok(mut v) => v.text().unwrap(),
        Err(e) => return Err(CError::from(e))
    };

    info!("(req {}, took: {})", path, TimeFormat(start.elapsed()));

    // catch State entity errors
    if value.contains(&String::from("No data for such identifier")) {
        return Err(CError::StateEntityError(value));
    }
    if value.contains(&String::from("Signing Error")) {
        return Err(CError::StateEntityError(value));
    }
    if value.contains(&String::from("Invalid sig hash - Odd number of characters.")) {
        return Err(CError::StateEntityError(value));
    }
    if value == String::from("User authorisation failed") {
        return Err(CError::StateEntityError(value));
    }
    if value == String::from("Error: Invalid o2, try again.") {
        return Err(CError::StateEntityError(value));
    }

    Ok(serde_json::from_str(value.as_str()).unwrap())
}
