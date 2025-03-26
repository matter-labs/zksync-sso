use alloy::primitives::U256;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod currency;
pub mod formatter;

pub use currency::Currency;
pub use formatter::MoneyFormatter;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Money {
    minor_value: U256,
    currency: Currency,
    extra_precision: Option<u8>,
}

impl Money {
    pub fn new(
        minor_value: U256,
        currency: Currency,
        extra_precision: Option<u8>,
    ) -> Self {
        Self { minor_value, currency, extra_precision }
    }

    pub fn eth(minor_value: U256) -> Self {
        println!("XDB - Money::eth - Creating with value={}", minor_value);
        Self::new(minor_value, Currency::eth(), None)
    }

    pub fn minor_value(&self) -> U256 {
        println!(
            "XDB - Money::minor_value - Getting value={}",
            self.minor_value
        );
        self.minor_value
    }

    pub fn decimals(&self) -> u8 {
        let total =
            self.currency.decimals() + self.extra_precision.unwrap_or(0);
        println!("XDB - Money::decimals - Calculation: currency={} + extra={:?} = {}",
            self.currency.decimals(),
            self.extra_precision,
            total
        );
        total
    }

    pub fn symbol(&self) -> &str {
        self.currency.symbol()
    }

    pub fn formatted_major(&self) -> String {
        let base = U256::from(10).pow(U256::from(self.decimals()));
        let major_part = self.minor_value / base;
        let minor_part = self.minor_value % base;
        format!("{}.{} {}", major_part, minor_part, self.symbol())
    }

    pub fn zero(currency: Currency) -> Self {
        Self::new(U256::ZERO, currency, None)
    }

    pub fn zero_eth() -> Self {
        Self::eth(U256::ZERO)
    }

    pub fn to_eth_wei(&self) -> Result<U256, &'static str> {
        if self.symbol() != "ETH" {
            return Err("Not an ETH amount");
        }
        Ok(self.minor_value)
    }
}

impl From<U256> for Money {
    fn from(value: U256) -> Self {
        Self::eth(value)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.minor_value, self.symbol())
    }
}

impl std::ops::Add for Money {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        assert_eq!(
            self.currency.decimals(),
            other.currency.decimals(),
            "Cannot add different currencies"
        );
        assert_eq!(
            self.currency.symbol(),
            other.currency.symbol(),
            "Cannot add different currencies"
        );

        Self::new(
            self.minor_value + other.minor_value,
            self.currency,
            self.extra_precision,
        )
    }
}

impl std::ops::Sub for Money {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        assert_eq!(
            self.currency.decimals(),
            other.currency.decimals(),
            "Cannot subtract different currencies"
        );
        assert_eq!(
            self.currency.symbol(),
            other.currency.symbol(),
            "Cannot subtract different currencies"
        );

        Self::new(
            self.minor_value - other.minor_value,
            self.currency,
            self.extra_precision,
        )
    }
}

impl Default for Money {
    fn default() -> Self {
        Self::new(U256::ZERO, Currency::usd(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_creation() {
        let value = U256::from(1_000_000_000_000_000_000u128);
        let money = Money::eth(value);

        assert_eq!(money.minor_value(), value);
        assert_eq!(money.decimals(), 18);
        assert_eq!(money.symbol(), "ETH");
    }

    #[test]
    fn test_from_u256() {
        let value = U256::from(1_000_000_000_000_000_000u128);
        let money: Money = value.into();

        assert_eq!(money.minor_value(), value);
        assert_eq!(money.decimals(), 18);
        assert_eq!(money.symbol(), "ETH");
    }

    #[test]
    fn test_arithmetic() {
        let one_eth = Money::eth(U256::from(1_000_000_000_000_000_000u128));
        let two_eth = Money::eth(U256::from(2_000_000_000_000_000_000u128));

        assert_eq!(one_eth.clone() + one_eth.clone(), two_eth.clone());
        assert_eq!(two_eth.clone() - one_eth.clone(), one_eth);
    }

    #[test]
    fn test_eth_conversion() {
        let eth = Money::eth(U256::from(1_000_000_000_000_000_000u128));
        assert!(eth.to_eth_wei().is_ok());

        let custom =
            Money::new(U256::from(100), Currency::new(2, "CUSTOM"), None);
        assert!(custom.to_eth_wei().is_err());
    }

    #[test]
    fn test_formatting() {
        let eth = Money::eth(U256::from(1_234_567_890_000_000_000u128));
        assert_eq!(eth.formatted_major(), "1.234567890000000000 ETH");
    }
}
