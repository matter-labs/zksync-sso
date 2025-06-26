use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Currency {
    decimals: u8,
    symbol: String,
}

impl Currency {
    pub fn eth() -> Self {
        Self { decimals: 18, symbol: "ETH".to_string() }
    }

    pub fn usd() -> Self {
        Self { decimals: 2, symbol: "USD".to_string() }
    }

    pub fn new(decimals: u8, symbol: impl Into<String>) -> Self {
        Self { decimals, symbol: symbol.into() }
    }

    pub fn decimals(&self) -> u8 {
        self.decimals
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Currency {
        pub fn jpy() -> Self {
            Self { decimals: 0, symbol: "JPY".to_string() }
        }

        pub fn mga() -> Self {
            Self { decimals: 1, symbol: "MGA".to_string() }
        }

        pub fn bhd() -> Self {
            Self { decimals: 3, symbol: "BHD".to_string() }
        }
    }
}
