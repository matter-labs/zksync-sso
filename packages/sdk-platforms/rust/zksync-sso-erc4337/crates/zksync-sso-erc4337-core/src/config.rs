pub mod contracts;

use contracts::Contracts;
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: Url,
    pub bundler_url: Url,
    pub contracts: Contracts,
}

impl Config {
    pub fn new(rpc_url: Url, bundler_url: Url, contracts: Contracts) -> Self {
        Self { rpc_url, bundler_url, contracts }
    }
}
