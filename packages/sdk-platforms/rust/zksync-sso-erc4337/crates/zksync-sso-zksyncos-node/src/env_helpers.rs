use std::env;

pub fn env_bool(key: &str) -> Option<bool> {
    env::var(key).ok().and_then(|v| match v.to_ascii_lowercase().as_str() {
        "1" | "true" | "on" => Some(true),
        "0" | "false" | "off" => Some(false),
        _ => None,
    })
}
