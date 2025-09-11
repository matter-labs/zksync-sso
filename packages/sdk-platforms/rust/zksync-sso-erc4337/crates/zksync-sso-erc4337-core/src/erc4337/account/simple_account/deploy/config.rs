const LOCAL_RPC_URL: &str = "http://localhost:8545";
const LOCAL_BUNDLER_URL: &str = "http://localhost:4337";
const LOCAL_PAYMASTER_URL: &str = "http://localhost:3000";

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub endpoints: Endpoints,
}

impl Config {
    pub fn local() -> Self {
        Config { endpoints: Endpoints::local() }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Endpoints {
    pub rpc: Endpoint,
    pub bundler: Endpoint,
    pub paymaster: Endpoint,
}

impl Endpoints {
    pub fn local() -> Self {
        Endpoints {
            rpc: Endpoint::local_rpc(),
            bundler: Endpoint::local_bundler(),
            paymaster: Endpoint::local_paymaster(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub base_url: String,
    pub api_key: String,
}

impl Endpoint {
    pub fn local_rpc() -> Self {
        Endpoint {
            base_url: LOCAL_RPC_URL.to_string(),
            api_key: "".to_string(),
        }
    }

    pub fn local_bundler() -> Self {
        Endpoint {
            base_url: LOCAL_BUNDLER_URL.to_string(),
            api_key: "".to_string(),
        }
    }

    pub fn local_paymaster() -> Self {
        Endpoint {
            base_url: LOCAL_PAYMASTER_URL.to_string(),
            api_key: "".to_string(),
        }
    }
}
