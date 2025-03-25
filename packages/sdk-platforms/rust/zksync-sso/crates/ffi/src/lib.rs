uniffi::setup_scaffolding!();

mod account;
mod config;
mod native_apis;

#[uniffi::export]
pub fn generate_random_challenge() -> String {
    let mut random_bytes = [0u8; 32];
    use rand::{thread_rng, Rng};
    thread_rng().fill(&mut random_bytes);

    base64::encode_engine(
        &random_bytes,
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
    )
}
