#![allow(async_fn_in_trait)]
use alloy::{primitives::TxHash, providers::Provider};
use alloy_zksync::network::{
    Zksync, receipt_response::ReceiptResponse as ZKReceiptResponse,
};
use eyre::{Result, eyre};
use log::debug;
use std::time::Duration;

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

pub trait ProviderExt: Provider<Zksync> + Clone {
    async fn wait_for_transaction_receipt(
        &self,
        tx_hash: TxHash,
    ) -> Result<ZKReceiptResponse> {
        self.wait_for_transaction_receipt_with_backoff(
            tx_hash,
            BackoffConfig::default(),
        )
        .await
    }

    async fn wait_for_transaction_receipt_with_backoff(
        &self,
        tx_hash: TxHash,
        config: BackoffConfig,
    ) -> Result<ZKReceiptResponse> {
        use rand::Rng;

        for attempt in 0..config.max_attempts {
            match self.get_transaction_receipt(tx_hash).await? {
                Some(receipt) => return Ok(receipt),
                None => {
                    debug!(
                        "Debug: Receipt not found, attempt {} of {}",
                        attempt + 1,
                        config.max_attempts
                    );

                    let base_delay = config.base_delay_ms as f64
                        * config.multiplier.powi(attempt as i32);
                    let capped_delay =
                        base_delay.min(config.max_delay_ms as f64) as u64;
                    let jitter = if config.jitter_ms > 0 {
                        rand::rng().random_range(0..=config.jitter_ms)
                    } else {
                        0
                    };
                    let total_delay = capped_delay + jitter;

                    debug!(
                        "Waiting {total_delay}ms (base: {capped_delay}ms, jitter: {jitter}ms) before retry"
                    );

                    tokio::time::sleep(Duration::from_millis(total_delay))
                        .await;
                }
            }
        }

        Err(eyre!(
            "Transaction receipt not found after {} attempts",
            config.max_attempts
        ))
    }
}

impl<P> ProviderExt for P where P: Provider<Zksync> + Clone {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::spawn_node_and_deploy_contracts;

    #[tokio::test]
    async fn test_wait_for_transaction_receipt_with_backoff() -> Result<()> {
        let (anvil_zksync, _, provider) =
            spawn_node_and_deploy_contracts().await?;

        let tx_hash = TxHash::from([1u8; 32]);

        let config = BackoffConfig {
            base_delay_ms: 50,
            max_delay_ms: 200,
            multiplier: 2.0,
            max_attempts: 3,
            jitter_ms: 10,
        };

        let start = std::time::Instant::now();
        let result = provider
            .wait_for_transaction_receipt_with_backoff(tx_hash, config)
            .await;
        let elapsed = start.elapsed();

        match result {
            Ok(_) => {
                panic!("Unexpected success: receipt found for dummy tx")
            }
            Err(e) => {
                debug!("Error: {e}");
                assert!(e.to_string().contains(
                    "Transaction receipt not found after 3 attempts"
                ));

                assert!(
                    elapsed.as_millis() >= 350,
                    "Expected at least 350ms elapsed (50 + 100 + 200), got {}ms",
                    elapsed.as_millis()
                );
                assert!(
                    elapsed.as_millis() <= 400,
                    "Expected at most 400ms elapsed (with jitter), got {}ms",
                    elapsed.as_millis()
                );
            }
        }

        drop(anvil_zksync);

        Ok(())
    }
}
