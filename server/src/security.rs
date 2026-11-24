use axum::http::{header, HeaderMap};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn random_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
pub fn token_hash(value: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(value.as_bytes()))
}
pub fn seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let values = headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|item| item.trim().split_once('='))
        .filter(|(key, _)| *key == name)
        .map(|(_, value)| value);
    let values: Vec<_> = values.collect();
    if values.len() == 1
        && values[0].len() == 43
        && values[0]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        Some(values[0].to_string())
    } else {
        None
    }
}
pub fn cookie(name: &str, value: &str, secure: bool, max_age: u64) -> String {
    format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{}",
        if secure { "; Secure" } else { "" }
    )
}
