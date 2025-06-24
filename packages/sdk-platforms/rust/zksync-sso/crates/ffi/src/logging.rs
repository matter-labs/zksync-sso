#[cfg(target_os = "android")]
use android_logger::Config;
use log::LevelFilter;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use oslog::OsLogger;

/// An enum representing the available verbosity level filters of the logger.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, uniffi::Enum,
)]
pub enum LogLevel {
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

/// Initialize the Android logger
#[uniffi::export]
#[allow(unused_variables)]
pub fn init_android_logger(level: LogLevel) {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        Config::default().with_max_level(level.into()).with_tag("Rust"),
    );
    #[cfg(not(target_os = "android"))]
    {} // No-op for non-Android targets
}

/// Initialize the Apple logger
#[uniffi::export]
#[allow(unused_variables)]
pub fn init_apple_logger(bundle_identifier: String, level: LogLevel) {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    OsLogger::new(&bundle_identifier)
        .level_filter(level.into())
        .init()
        .unwrap();
    #[cfg(not(any(target_os = "ios", target_os = "macos")))]
    {} // No-op for non-Apple targets
}
