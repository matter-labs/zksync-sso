use crate::utils::alloy_utilities::test_utilities::node_backend::{
    TestNodeBackend, resolve_test_node_backend,
};

pub const DEFAULT_ANVIL_KEY: &str =
    "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

pub const DEFAULT_ZKSYNC_OS_KEY: &str =
    "0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110";
pub const DEFAULT_RICH_WALLET_9_KEY: &str =
    "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6";

#[derive(Debug, Clone)]
pub struct TestInfraConfig {
    pub signer_private_key: String,
}

impl TestInfraConfig {
    pub fn rich_wallet_9() -> Self {
        Self { signer_private_key: DEFAULT_RICH_WALLET_9_KEY.to_string() }
    }
}

impl Default for TestInfraConfig {
    fn default() -> Self {
        let signer_private_key = match resolve_test_node_backend() {
            TestNodeBackend::ZkSyncOs => DEFAULT_ZKSYNC_OS_KEY,
            TestNodeBackend::Anvil => DEFAULT_ANVIL_KEY,
        };

        Self { signer_private_key: signer_private_key.to_string() }
    }
}
