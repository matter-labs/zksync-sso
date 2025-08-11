use crate::utils::anvil_zksync::rich_wallet::RichWallet;
use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use alloy_zksync::wallet::ZksyncWallet;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployWallet {
    pub private_key_hex: String,
}

impl DeployWallet {
    pub fn new(private_key_hex: String) -> eyre::Result<Self> {
        _ = PrivateKeySigner::from_str(&private_key_hex)?;
        Ok(Self { private_key_hex })
    }

    pub fn random() -> Self {
        let signer = PrivateKeySigner::random();
        let private_key_hex = alloy::hex::encode(signer.to_bytes());
        Self { private_key_hex }
    }

    pub fn rich_wallet() -> Self {
        let rich_wallet = RichWallet::four();
        let private_key_hex = rich_wallet.private_key_hex().to_string();
        Self { private_key_hex }
    }

    pub fn address(&self) -> Address {
        let signer = PrivateKeySigner::from_str(&self.private_key_hex).unwrap();
        let wallet = ZksyncWallet::from(signer);
        wallet.default_signer().address()
    }
}

impl TryFrom<&str> for DeployWallet {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_string())
    }
}

impl TryFrom<String> for DeployWallet {
    type Error = eyre::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random() -> eyre::Result<()> {
        let deploy_wallet = DeployWallet::random();
        println!("Deploy wallet: {deploy_wallet:?}");

        let signer =
            PrivateKeySigner::from_str(&deploy_wallet.private_key_hex)?;
        let wallet = ZksyncWallet::from(signer);
        let address = wallet.default_signer().address();
        eyre::ensure!(address == deploy_wallet.address(), "Address mismatch");

        Ok(())
    }
}
