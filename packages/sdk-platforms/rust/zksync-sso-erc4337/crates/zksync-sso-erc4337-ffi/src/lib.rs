#[uniffi::export]
pub fn initialize_logging() -> Result<(), String> {
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Info)
                .with_tag("zksync-sso-erc4337"),
        );
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    {
        oslog::OsLogger::new("com.matterlabs.zksync-sso-erc4337")
            .level_filter(log::LevelFilter::Info)
            .init()
            .map_err(|e| format!("Failed to initialize logging: {}", e))?;
    }

    #[cfg(not(any(
        target_os = "android",
        target_os = "ios",
        target_os = "macos"
    )))]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
            .init();
    }

    Ok(())
}

// Generate the UniFFI scaffolding
uniffi::setup_scaffolding!();
