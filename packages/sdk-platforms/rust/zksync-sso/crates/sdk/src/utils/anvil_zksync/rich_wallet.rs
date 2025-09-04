use alloy::{
    primitives::Address,
    signers::local::{LocalSignerError, PrivateKeySigner},
};
use alloy_zksync::wallet::ZksyncWallet;
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::LazyLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RichWallet {
    private_key_hex: String,
    address: Address,
}

impl RichWallet {
    fn from_private_key(
        private_key_hex: String,
    ) -> Result<Self, LocalSignerError> {
        let signer = PrivateKeySigner::from_str(&private_key_hex)?;
        let wallet = ZksyncWallet::from(signer);
        let address = wallet.default_signer().address();

        Ok(Self { private_key_hex, address })
    }

    #[allow(dead_code)]
    pub fn private_key_hex(&self) -> &str {
        &self.private_key_hex
    }

    #[allow(dead_code)]
    pub fn address(&self) -> Address {
        self.address
    }

    #[allow(dead_code)]
    pub fn to_zksync_wallet(&self) -> Result<ZksyncWallet, LocalSignerError> {
        let signer = self.to_local_signer()?;
        Ok(ZksyncWallet::from(signer))
    }

    #[allow(dead_code)]
    pub fn to_local_signer(
        &self,
    ) -> Result<PrivateKeySigner, LocalSignerError> {
        PrivateKeySigner::from_str(&self.private_key_hex)
    }

    // Convenience methods to access specific rich wallets
    /// Get the zeroth rich wallet (index 0)
    /// Address: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
    #[allow(dead_code)]
    pub fn zero() -> &'static RichWallet {
        &RICH_WALLETS[0]
    }

    /// Get the first rich wallet (index 1)
    /// Address: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
    #[allow(dead_code)]
    pub fn one() -> &'static RichWallet {
        &RICH_WALLETS[1]
    }

    /// Get the second rich wallet (index 2)
    /// Address: 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
    #[allow(dead_code)]
    pub fn two() -> &'static RichWallet {
        &RICH_WALLETS[2]
    }

    /// Get the third rich wallet (index 3)
    /// Address: 0x90F79bf6EB2c4f870365E785982E1f101E93b906
    #[allow(dead_code)]
    pub fn three() -> &'static RichWallet {
        &RICH_WALLETS[3]
    }

    /// Get the fourth rich wallet (index 4)
    /// Address: 0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65
    #[allow(dead_code)]
    pub fn four() -> &'static RichWallet {
        &RICH_WALLETS[4]
    }

    /// Get the fifth rich wallet (index 5)
    /// Address: 0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc
    #[allow(dead_code)]
    pub fn five() -> &'static RichWallet {
        &RICH_WALLETS[5]
    }

    /// Get the sixth rich wallet (index 6)
    /// Address: 0x976EA74026E726554dB657fA54763abd0C3a0aa9
    #[allow(dead_code)]
    pub fn six() -> &'static RichWallet {
        &RICH_WALLETS[6]
    }

    /// Get the seventh rich wallet (index 7)
    /// Address: 0x14dC79964da2C08b23698B3D3cc7Ca32193d9955
    #[allow(dead_code)]
    pub fn seven() -> &'static RichWallet {
        &RICH_WALLETS[7]
    }

    /// Get the eighth rich wallet (index 8)
    /// Address: 0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f
    #[allow(dead_code)]
    pub fn eight() -> &'static RichWallet {
        &RICH_WALLETS[8]
    }

    /// Get the ninth rich wallet (index 9)
    /// Address: 0xa0Ee7A142d267C1f36714E4a8F75612F20a79720
    #[allow(dead_code)]
    pub fn nine() -> &'static RichWallet {
        &RICH_WALLETS[9]
    }
}

impl TryFrom<String> for RichWallet {
    type Error = LocalSignerError;

    fn try_from(private_key_hex: String) -> Result<Self, Self::Error> {
        Self::from_private_key(private_key_hex)
    }
}

impl TryFrom<&str> for RichWallet {
    type Error = LocalSignerError;

    fn try_from(private_key_hex: &str) -> Result<Self, Self::Error> {
        Self::try_from(private_key_hex.to_string())
    }
}

/// Static array of rich wallets for testing
///
/// Rich Accounts (each with 10000.000000000000000000 ETH)
/// ========================
/// (0) 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
/// (1) 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
/// (2) 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
/// (3) 0x90F79bf6EB2c4f870365E785982E1f101E93b906
/// (4) 0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65
/// (5) 0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc
/// (6) 0x976EA74026E726554dB657fA54763abd0C3a0aa9
/// (7) 0x14dC79964da2C08b23698B3D3cc7Ca32193d9955
/// (8) 0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f
/// (9) 0xa0Ee7A142d267C1f36714E4a8F75612F20a79720
///
/// Mnemonic: test test test test test test test test test test test junk
/// Derivation path: m/44'/60'/0'/0/0
/// Chain ID: 260
pub static RICH_WALLETS: LazyLock<[RichWallet; 10]> = LazyLock::new(|| {
    [
        RichWallet::try_from(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        ).unwrap(),
        RichWallet::try_from(
            "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d",
        ).unwrap(),
        RichWallet::try_from(
            "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a",
        ).unwrap(),
        RichWallet::try_from(
            "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6",
        ).unwrap(),
        RichWallet::try_from(
            "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a",
        ).unwrap(),
        RichWallet::try_from(
            "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba",
        ).unwrap(),
        RichWallet::try_from(
            "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e",
        ).unwrap(),
        RichWallet::try_from(
            "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356",
        ).unwrap(),
        RichWallet::try_from(
            "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97",
        ).unwrap(),
        RichWallet::try_from(
            "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
        ).unwrap(),
    ]
});

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_rich_wallets_derive_correct_addresses() {
        // Expected addresses from the comments above
        let expected_addresses = [
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", // (0)
            "0x70997970C51812dc3A010C7d01b50e0d17dc79C8", // (1)
            "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", // (2)
            "0x90F79bf6EB2c4f870365E785982E1f101E93b906", // (3)
            "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65", // (4)
            "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc", // (5)
            "0x976EA74026E726554dB657fA54763abd0C3a0aa9", // (6)
            "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955", // (7)
            "0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f", // (8)
            "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720", // (9)
        ];

        // Test that each rich wallet derives the correct address
        for (i, expected_address) in expected_addresses.iter().enumerate() {
            let actual_address = format!("{:#x}", RICH_WALLETS[i].address());
            assert_eq!(
                actual_address,
                expected_address.to_lowercase(),
                "Rich wallet {i} address mismatch. Expected: {expected_address}, Got: {actual_address}"
            );
        }

        // Also verify we have exactly 10 wallets
        assert_eq!(RICH_WALLETS.len(), 10);
    }

    #[test]
    fn test_rich_wallet_convenience_methods() {
        // Test that all convenience methods return the correct wallets
        let expected_addresses = [
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", // zero
            "0x70997970C51812dc3A010C7d01b50e0d17dc79C8", // one
            "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", // two
            "0x90F79bf6EB2c4f870365E785982E1f101E93b906", // three
            "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65", // four
            "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc", // five
            "0x976EA74026E726554dB657fA54763abd0C3a0aa9", // six
            "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955", // seven
            "0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f", // eight
            "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720", // nine
        ];

        // Test each convenience method
        assert_eq!(
            format!("{:#x}", RichWallet::zero().address()),
            expected_addresses[0].to_lowercase()
        );
        assert_eq!(
            format!("{:#x}", RichWallet::one().address()),
            expected_addresses[1].to_lowercase()
        );
        assert_eq!(
            format!("{:#x}", RichWallet::two().address()),
            expected_addresses[2].to_lowercase()
        );
        assert_eq!(
            format!("{:#x}", RichWallet::three().address()),
            expected_addresses[3].to_lowercase()
        );
        assert_eq!(
            format!("{:#x}", RichWallet::four().address()),
            expected_addresses[4].to_lowercase()
        );
        assert_eq!(
            format!("{:#x}", RichWallet::five().address()),
            expected_addresses[5].to_lowercase()
        );
        assert_eq!(
            format!("{:#x}", RichWallet::six().address()),
            expected_addresses[6].to_lowercase()
        );
        assert_eq!(
            format!("{:#x}", RichWallet::seven().address()),
            expected_addresses[7].to_lowercase()
        );
        assert_eq!(
            format!("{:#x}", RichWallet::eight().address()),
            expected_addresses[8].to_lowercase()
        );
        assert_eq!(
            format!("{:#x}", RichWallet::nine().address()),
            expected_addresses[9].to_lowercase()
        );

        // Verify they're the same as accessing via array index
        assert_eq!(RichWallet::zero().address(), RICH_WALLETS[0].address());
        assert_eq!(RichWallet::two().address(), RICH_WALLETS[2].address());
        assert_eq!(RichWallet::nine().address(), RICH_WALLETS[9].address());
    }

    #[test]
    fn test_rich_wallet_to_zksync_wallet() {
        let rich_wallet = RichWallet::zero();
        let zksync_wallet = rich_wallet.to_zksync_wallet().unwrap();

        // Verify the wallet has the correct address
        assert_eq!(
            zksync_wallet.default_signer().address(),
            rich_wallet.address()
        );

        // Test with a few more wallets
        let second_wallet = RichWallet::one();
        let second_zksync = second_wallet.to_zksync_wallet().unwrap();
        assert_eq!(
            second_zksync.default_signer().address(),
            second_wallet.address()
        );

        let tenth_wallet = RichWallet::nine();
        let tenth_zksync = tenth_wallet.to_zksync_wallet().unwrap();
        assert_eq!(
            tenth_zksync.default_signer().address(),
            tenth_wallet.address()
        );
    }

    #[test]
    fn test_rich_wallet_to_local_signer() {
        let rich_wallet = RichWallet::zero();
        let local_signer = rich_wallet.to_local_signer().unwrap();

        // Verify the signer has the correct address
        assert_eq!(local_signer.address(), rich_wallet.address());

        // Test with a few more wallets
        let third_wallet = RichWallet::two();
        let third_signer = third_wallet.to_local_signer().unwrap();
        assert_eq!(third_signer.address(), third_wallet.address());

        let ninth_wallet = RichWallet::eight();
        let ninth_signer = ninth_wallet.to_local_signer().unwrap();
        assert_eq!(ninth_signer.address(), ninth_wallet.address());
    }

    #[test]
    fn test_rich_wallet_from_private_key() {
        let private_key = "0x5de4111afa1a4b94908f83103c3a57e0c3c9e9da2dd5a02a84e9fde30d7e96c3";
        let rich_wallet =
            RichWallet::from_private_key(private_key.to_string()).unwrap();
        assert_eq!(
            rich_wallet.address(),
            address!("0x6a34Ea49c29BF7Cce95F51E7F0f419831Ad5dBC6")
        );
    }
}
