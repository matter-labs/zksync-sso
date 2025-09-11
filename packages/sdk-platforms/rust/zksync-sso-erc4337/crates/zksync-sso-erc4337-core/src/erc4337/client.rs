use crate::{
    config::Config,
    erc4337::{send_call::SendCalls, signature::sign_user_operation_dummy},
    result::Result,
};
use alloy::{
    primitives::Bytes, providers::Provider,
    rpc::types::erc4337::SendUserOperation, signers::local::PrivateKeySigner,
};
use alloy_provider::ext::Erc4337Api;

#[cfg(test)]
pub mod test_utils;

#[derive(Debug, Clone)]
pub struct Client<P> {
    pub config: Config,
    pub signer: PrivateKeySigner,
    pub provider: P,
}

impl<P> Client<P>
where
    P: Provider + Clone,
{
    pub fn new(config: Config, signer: PrivateKeySigner, provider: P) -> Self {
        Self { config, signer, provider }
    }

    pub async fn send_user_operation(&self, req: SendCalls) -> Result<Bytes> {
        let signed = sign_user_operation_dummy(&self.config, &self.signer, req);

        let resp = self
            .provider
            .send_user_operation(
                SendUserOperation::EntryPointV07(signed.packed),
                self.config.contracts.entry_point,
            )
            .await
            .map_err(|e| {
                crate::error::ZkSyncSsoError::SendUserOperation(e.to_string())
            })?;

        Ok(resp.user_op_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::contracts::Contracts;
    use alloy::{
        primitives::{Address, Bytes, U256},
        providers::{Provider, ProviderBuilder, WalletProvider},
        signers::{Signer, local::PrivateKeySigner},
    };
    use eyre::Result;
    use std::str::FromStr;

    type ProviderType = alloy_provider::fillers::FillProvider<
        alloy_provider::fillers::JoinFill<
            alloy_provider::fillers::JoinFill<
                alloy_provider::Identity,
                alloy_provider::fillers::JoinFill<
                    alloy_provider::fillers::GasFiller,
                    alloy_provider::fillers::JoinFill<
                        alloy_provider::fillers::BlobGasFiller,
                        alloy_provider::fillers::JoinFill<
                            alloy_provider::fillers::NonceFiller,
                            alloy_provider::fillers::ChainIdFiller,
                        >,
                    >,
                >,
            >,
            alloy_provider::fillers::WalletFiller<
                alloy::network::EthereumWallet,
            >,
        >,
        alloy_provider::layers::AnvilProvider<alloy_provider::RootProvider>,
    >;

    fn create_test_config() -> Config {
        let contracts = Contracts::new(
            Address::from_str("0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789")
                .unwrap(), // EntryPoint v0.7
            Address::from_str("0x9406Cc6185a346906296840746125a0E44976454")
                .unwrap(), // AccountFactory
        );

        Config::new(
            "http://localhost:8545".to_string(),
            "http://localhost:8545".to_string(), // Using anvil as both RPC and bundler
            contracts,
        )
    }

    fn create_test_signer() -> PrivateKeySigner {
        // Use a well-known test private key
        let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        PrivateKeySigner::from_str(private_key).unwrap()
    }

    fn create_test_send_calls() -> SendCalls {
        SendCalls {
            account: Address::from_str(
                "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            )
            .unwrap(),
            calls: vec![crate::erc4337::send_call::Call {
                to: Address::from_str(
                    "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
                )
                .unwrap(),
                data: Bytes::from_static(b"test_data"),
                value: U256::from(1000),
            }],
        }
    }

    #[tokio::test]
    async fn test_client_creation() {
        let provider = ProviderBuilder::new().connect_anvil_with_wallet();
        let config = create_test_config();
        let signer = create_test_signer();

        let client = Client::new(config, signer, provider);

        // Test that client was created successfully
        assert_eq!(
            client.config.contracts.entry_point,
            Address::from_str("0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789")
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_send_user_operation() {
        let provider = ProviderBuilder::new().connect_anvil_with_wallet();
        let config = create_test_config();
        let signer = create_test_signer();
        let send_calls = create_test_send_calls();

        let client = Client::new(config, signer, provider);

        // Test sending user operation
        // Note: This will fail with the dummy signature, but we can test the flow
        let result = client.send_user_operation(send_calls).await;

        // The result should be an error due to dummy signature, but the client should handle it gracefully
        assert!(result.is_err());

        // Verify the error is related to sending user operation (as expected with dummy signature)
        match result {
            Err(crate::error::ZkSyncSsoError::SendUserOperation(_)) => {
                // This is expected with dummy signature
            }
            _ => panic!("Expected SendUserOperation error, got: {:?}", result),
        }
    }

    // #[tokio::test]
    // async fn test_client_with_different_providers() {
    //     let (_anvil, _bundler_url) = start_bundler().await;
    //     let config = create_test_config();
    //     let signer = create_test_signer();

    //     // Test with anvil provider
    //     let anvil_provider = ProviderBuilder::new().connect_anvil_with_wallet();
    //     let client1 = Client::new(config.clone(), signer.clone(), anvil_provider);

    //     // Test with HTTP provider
    //     let http_provider = ProviderBuilder::new().connect_http("http://localhost:8545");
    //     let client2 = Client::new(config, signer, http_provider);

    //     // Both clients should be created successfully
    //     assert_eq!(
    //         client1.config.contracts.entry_point,
    //         client2.config.contracts.entry_point
    //     );
    // }

    // #[tokio::test]
    // async fn test_multiple_anvil_instances() {
    //     // Test that we can start multiple anvil instances for different tests
    //     let (_anvil1, url1) = start_bundler().await;
    //     let (_anvil2, url2) = start_bundler().await;

    //     // URLs should be different (different ports)
    //     assert_ne!(url1, url2);

    //     // Both should be valid HTTP URLs
    //     assert!(url1.starts_with("http://localhost:"));
    //     assert!(url2.starts_with("http://localhost:"));
    // }

    // #[tokio::test]
    // async fn test_error_types() {
    //     let (_anvil, _bundler_url) = start_bundler().await;
    //     let config = create_test_config();
    //     let signer = create_test_signer();
    //     let send_calls = create_test_send_calls();

    //     let provider = ProviderBuilder::new().connect_anvil_with_wallet();
    //     let client = Client::new(config, signer, provider);

    //     // Test that we get the correct error type
    //     let result = client.send_user_operation(send_calls).await;

    //     match result {
    //         Err(crate::error::ZkSyncSsoError::SendUserOperation(error_msg)) => {
    //             // Verify the error message contains useful information
    //             assert!(!error_msg.is_empty());
    //             println!("SendUserOperation error: {}", error_msg);
    //         }
    //         Err(other_error) => {
    //             panic!("Expected SendUserOperation error, got: {:?}", other_error);
    //         }
    //         Ok(_) => {
    //             panic!("Expected an error due to dummy signature");
    //         }
    //     }
    // }

    // #[test]
    // fn test_error_enum_variants() {
    //     // Test that all error variants can be created and formatted
    //     let config_error =
    //         crate::error::ZkSyncSsoError::InvalidConfiguration("test config error".to_string());
    //     let send_error =
    //         crate::error::ZkSyncSsoError::SendUserOperation("test send error".to_string());
    //     let validation_error = crate::error::ZkSyncSsoError::UserOperationValidation(
    //         "test validation error".to_string(),
    //     );
    //     let provider_error =
    //         crate::error::ZkSyncSsoError::ProviderError("test provider error".to_string());
    //     let signature_error =
    //         crate::error::ZkSyncSsoError::SignatureError("test signature error".to_string());

    //     // Test error formatting
    //     assert!(config_error.to_string().contains("Invalid configuration"));
    //     assert!(
    //         send_error
    //             .to_string()
    //             .contains("Failed to send user operation")
    //     );
    //     assert!(
    //         validation_error
    //             .to_string()
    //             .contains("User operation validation failed")
    //     );
    //     assert!(provider_error.to_string().contains("Provider error"));
    //     assert!(signature_error.to_string().contains("Signature error"));

    //     // Test error messages
    //     assert!(config_error.to_string().contains("test config error"));
    //     assert!(send_error.to_string().contains("test send error"));
    //     assert!(
    //         validation_error
    //             .to_string()
    //             .contains("test validation error")
    //     );
    //     assert!(provider_error.to_string().contains("test provider error"));
    //     assert!(signature_error.to_string().contains("test signature error"));
    // }
}
