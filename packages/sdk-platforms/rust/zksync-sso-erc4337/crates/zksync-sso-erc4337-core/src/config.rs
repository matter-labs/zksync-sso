pub mod contracts;

use contracts::Contracts;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub bundler_url: String,
    pub contracts: Contracts,
}

impl Config {
    pub fn new(
        rpc_url: String,
        bundler_url: String,
        contracts: Contracts,
    ) -> Self {
        Self { rpc_url, bundler_url, contracts }
    }
}
