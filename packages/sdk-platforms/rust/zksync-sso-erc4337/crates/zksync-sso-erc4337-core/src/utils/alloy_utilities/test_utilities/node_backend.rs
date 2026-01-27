use std::{env, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestNodeBackend {
    #[default]
    Anvil,
    ZkSyncOs,
}

impl FromStr for TestNodeBackend {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "anvil" => Ok(Self::Anvil),
            "zksyncos" => Ok(Self::ZkSyncOs),
            other => {
                Err(eyre::eyre!("Unsupported test node backend value: {other}"))
            }
        }
    }
}

pub fn resolve_test_node_backend() -> TestNodeBackend {
    env::var("SSO_TEST_NODE_BACKEND")
        .ok()
        .and_then(|raw| TestNodeBackend::from_str(&raw).ok())
        .unwrap_or_default()
}
