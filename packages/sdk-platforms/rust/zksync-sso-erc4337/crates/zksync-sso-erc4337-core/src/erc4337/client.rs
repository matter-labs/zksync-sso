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

        let alloy_packed = signed.packed.into();

        let resp = self
            .provider
            .send_user_operation(
                SendUserOperation::EntryPointV07(alloy_packed),
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
        providers::ProviderBuilder,
        signers::local::PrivateKeySigner,
    };
    use std::str::FromStr;

    fn create_test_config() -> Config {
        let contracts = Contracts::new(
            Address::from_str("0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789")
                .unwrap(),
            Address::from_str("0x9406Cc6185a346906296840746125a0E44976454")
                .unwrap(),
        );

        Config::new(
            "http://localhost:8545".parse().unwrap(),
            "http://localhost:4337".parse().unwrap(),
            contracts,
        )
    }

    fn create_test_signer() -> PrivateKeySigner {
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
        let result = client.send_user_operation(send_calls).await;
        assert!(result.is_err());

        match result {
            Err(crate::error::ZkSyncSsoError::SendUserOperation(_)) => {
                // This is expected with dummy signature
            }
            _ => panic!("Expected SendUserOperation error, got: {:?}", result),
        }
    }
}
