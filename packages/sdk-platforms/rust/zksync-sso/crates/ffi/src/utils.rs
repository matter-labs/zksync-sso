#[cfg(target_os = "android")]
use android_logger::Config;
use log::LevelFilter;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use oslog::OsLogger;

#[uniffi::export]
/// Initialize the Android logger
pub fn init_android_logger() {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        Config::default().with_max_level(LevelFilter::Trace).with_tag("Rust"),
    );
    #[cfg(not(target_os = "android"))]
    {} // No-op for non-Android targets
}

#[uniffi::export]
/// Initialize the Apple logger
#[allow(unused_variables)]
pub fn init_apple_logger(bundle_identifier: String) {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    OsLogger::new(&bundle_identifier)
        .level_filter(LevelFilter::Trace)
        .init()
        .unwrap();
    #[cfg(not(any(target_os = "ios", target_os = "macos")))]
    {} // No-op for non-Apple targets
}
