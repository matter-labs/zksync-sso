uniffi::setup_scaffolding!();

mod account;
mod config;
mod logging;
mod native_apis;
mod utils;

use base64::Engine;
use rand::{Rng, rng};

#[uniffi::export]
pub fn generate_random_challenge() -> String {
    let mut random_bytes = [0u8; 32];
    rng().fill(&mut random_bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes)
}
