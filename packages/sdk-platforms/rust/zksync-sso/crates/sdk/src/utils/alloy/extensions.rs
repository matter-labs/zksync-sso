#![allow(async_fn_in_trait)]
use alloy::{primitives::TxHash, providers::Provider};
use alloy_zksync::network::{
    Zksync, receipt_response::ReceiptResponse as ZKReceiptResponse,
};
use eyre::{Result, eyre};
use std::time::Duration;

pub trait ProviderExt: Provider<Zksync> + Clone {
    async fn wait_for_transaction_receipt(
        &self,
        tx_hash: TxHash,
    ) -> Result<ZKReceiptResponse> {
        self.wait_for_transaction_receipt_with_interval_and_max_attempts(
            tx_hash, 1000, 3,
        )
        .await
    }

    async fn wait_for_transaction_receipt_with_interval_and_max_attempts(
        &self,
        tx_hash: TxHash,
        interval_ms: u64,
        max_attempts: u32,
    ) -> Result<ZKReceiptResponse> {
        for attempt in 0..max_attempts {
            match self.get_transaction_receipt(tx_hash).await? {
                Some(receipt) => return Ok(receipt),
                None => {
                    println!(
                        "Debug: Receipt not found, attempt {} of {}",
                        attempt + 1,
                        max_attempts
                    );
                    tokio::time::sleep(Duration::from_millis(interval_ms))
                        .await;
                }
            }
        }

        Err(eyre!(
            "Transaction receipt not found after {} attempts",
            max_attempts
        ))
    }
}

impl<P> ProviderExt for P where P: Provider<Zksync> + Clone {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_utils::spawn_node_and_deploy_contracts;

    #[tokio::test]
    async fn test_wait_for_transaction_receipt() -> Result<()> {
        let (anvil_zksync, _, provider) =
            spawn_node_and_deploy_contracts().await?;

        // Create a dummy transaction hash (this test will fail but demonstrates usage)
        let tx_hash = TxHash::from([1u8; 32]);

        let result = provider
            .wait_for_transaction_receipt_with_interval_and_max_attempts(
                tx_hash, 100, 2,
            )
            .await;

        match result {
            Ok(_) => {
                assert!(false, "Unexpected success: receipt found for dummy tx")
            }
            Err(e) => {
                println!("Error: {}", e);
                assert!(
                    e.to_string()
                        .contains("Transaction receipt not found after")
                );
                println!("Expected error received: {}", e);
            }
        }

        drop(anvil_zksync);

        Ok(())
    }
}
