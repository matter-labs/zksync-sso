pub mod client;
pub mod config;
pub mod models;
pub mod pimlico;

use async_trait::async_trait;
use models::receipt::UserOperationReceipt;

#[derive(Debug, Clone)]
pub struct BackoffConfig {
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
    pub max_attempts: u32,
    pub jitter_ms: u64,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            base_delay_ms: 1000,
            max_delay_ms: 10_000,
            multiplier: 2.0,
            max_attempts: 5,
            jitter_ms: 100,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
pub trait Bundler {
    async fn get_user_operation_receipt(
        &self,
        hash: String,
    ) -> eyre::Result<Option<UserOperationReceipt>>;

    async fn wait_for_user_operation_receipt(
        &self,
        hash: String,
    ) -> eyre::Result<UserOperationReceipt> {
        self.wait_for_user_operation_receipt_with_backoff(
            hash,
            BackoffConfig::default(),
        )
        .await
    }

    async fn wait_for_user_operation_receipt_with_backoff(
        &self,
        hash: String,
        config: BackoffConfig,
    ) -> eyre::Result<UserOperationReceipt> {
        use rand::Rng;

        for attempt in 0..config.max_attempts {
            match self.get_user_operation_receipt(hash.clone()).await? {
                Some(receipt) => return Ok(receipt),
                None => {
                    println!(
                        "Receipt not found, attempt {} of {}",
                        attempt + 1,
                        config.max_attempts
                    );

                    let base_delay = config.base_delay_ms as f64
                        * config.multiplier.powi(attempt as i32);
                    let capped_delay =
                        base_delay.min(config.max_delay_ms as f64) as u64;
                    let jitter = if config.jitter_ms > 0 {
                        rand::thread_rng().gen_range(0..=config.jitter_ms)
                    } else {
                        0
                    };
                    let total_delay = capped_delay + jitter;

                    println!(
                        "Waiting {total_delay}ms (base: {capped_delay}ms, jitter: {jitter}ms) before retry"
                    );

                    // Sleep using the appropriate mechanism for the target
                    #[cfg(all(
                        target_arch = "wasm32",
                        not(feature = "tokio-runtime")
                    ))]
                    {
                        // For WASM without tokio, use a simple async sleep
                        use js_sys::Promise;
                        use wasm_bindgen_futures::JsFuture;
                        use web_sys::window;

                        let promise = Promise::new(&mut |resolve, _| {
                            window()
                                .unwrap()
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    &resolve,
                                    total_delay as i32,
                                )
                                .unwrap();
                        });
                        let _ = JsFuture::from(promise).await;
                    }

                    #[cfg(all(
                        not(target_arch = "wasm32"),
                        feature = "tokio-runtime"
                    ))]
                    {
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            total_delay,
                        ))
                        .await;
                    }
                }
            }
        }

        Err(eyre::eyre!(
            "User operation receipt not found after {} attempts",
            config.max_attempts
        ))
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
pub trait Bundler {
    async fn get_user_operation_receipt(
        &self,
        hash: String,
    ) -> eyre::Result<Option<UserOperationReceipt>>;

    async fn wait_for_user_operation_receipt(
        &self,
        hash: String,
    ) -> eyre::Result<UserOperationReceipt> {
        self.wait_for_user_operation_receipt_with_backoff(
            hash,
            BackoffConfig::default(),
        )
        .await
    }

    async fn wait_for_user_operation_receipt_with_backoff(
        &self,
        hash: String,
        config: BackoffConfig,
    ) -> eyre::Result<UserOperationReceipt> {
        use rand::Rng;

        for attempt in 0..config.max_attempts {
            match self.get_user_operation_receipt(hash.clone()).await? {
                Some(receipt) => return Ok(receipt),
                None => {
                    println!(
                        "Receipt not found, attempt {} of {}",
                        attempt + 1,
                        config.max_attempts
                    );

                    let base_delay = config.base_delay_ms as f64
                        * config.multiplier.powi(attempt as i32);
                    let capped_delay =
                        base_delay.min(config.max_delay_ms as f64) as u64;
                    let jitter = if config.jitter_ms > 0 {
                        rand::thread_rng().gen_range(0..=config.jitter_ms)
                    } else {
                        0
                    };
                    let total_delay = capped_delay + jitter;

                    println!(
                        "Waiting {total_delay}ms (base: {capped_delay}ms, jitter: {jitter}ms) before retry"
                    );

                    // Sleep using the appropriate mechanism for the target
                    #[cfg(all(
                        target_arch = "wasm32",
                        not(feature = "tokio-runtime")
                    ))]
                    {
                        // For WASM without tokio, use a simple async sleep
                        use js_sys::Promise;
                        use wasm_bindgen_futures::JsFuture;
                        use web_sys::window;

                        let promise = Promise::new(&mut |resolve, _| {
                            window()
                                .unwrap()
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    &resolve,
                                    total_delay as i32,
                                )
                                .unwrap();
                        });
                        let _ = JsFuture::from(promise).await;
                    }

                    #[cfg(all(
                        not(target_arch = "wasm32"),
                        feature = "tokio-runtime"
                    ))]
                    {
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            total_delay,
                        ))
                        .await;
                    }
                }
            }
        }

        Err(eyre::eyre!(
            "User operation receipt not found after {} attempts",
            config.max_attempts
        ))
    }
}
