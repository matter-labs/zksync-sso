pub mod contracts;

use crate::chain::Chain;
use contracts::Contracts;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rpc_url: Url,
    pub bundler_url: Url,
    pub chain: Chain,
    pub contracts: Contracts,
}

impl Config {
    pub fn new(
        rpc_url: Url,
        bundler_url: Url,
        chain: Chain,
        contracts: Contracts,
    ) -> Self {
        Self { rpc_url, bundler_url, chain, contracts }
    }
}
