use alloy::node_bindings::AnvilInstance;
use zksync_sso_zksyncos_node::instance::ZkSyncOsInstance;

#[derive(Debug)]
pub enum TestNodeHandle {
    Anvil(AnvilInstance),
    ZkSyncOs(ZkSyncOsInstance),
}

impl TestNodeHandle {
    pub fn variant_name(&self) -> &'static str {
        match self {
            Self::Anvil(_) => "anvil",
            Self::ZkSyncOs(_) => "zksyncos",
        }
    }

    pub fn as_anvil(&self) -> Option<&AnvilInstance> {
        match self {
            Self::Anvil(instance) => Some(instance),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn as_zksync_os(&self) -> Option<&ZkSyncOsInstance> {
        match self {
            Self::ZkSyncOs(instance) => Some(instance),
            _ => None,
        }
    }
}
