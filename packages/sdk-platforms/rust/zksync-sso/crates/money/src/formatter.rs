use super::Money;
use fixed_decimal::FixedDecimal;
use icu::decimal::{
    options::FixedDecimalFormatterOptions, provider::Baked,
    FixedDecimalFormatter,
};
use icu::locid::Locale;
use std::str::FromStr;

pub struct MoneyFormatter {
    pub locale: Locale,
    pub decimal_formatter: FixedDecimalFormatter,
    pub display_decimals: Option<u8>,
}

impl Default for MoneyFormatter {
    fn default() -> Self {
        match Self::new("en-US") {
            Ok(formatter) => formatter,
            Err(e) => panic!("Failed to create default formatter: {}", e),
        }
    }
}

impl MoneyFormatter {
    pub fn new(locale_id: &str) -> Result<Self, String> {
        let locale: Locale =
            locale_id.parse().map_err(|e| format!("Invalid locale: {}", e))?;

        let options = FixedDecimalFormatterOptions::default();

        let provider = Baked;
        let decimal_formatter = FixedDecimalFormatter::try_new_unstable(
            &provider,
            &locale.clone().into(),
            options,
        )
        .map_err(|e| format!("Failed to create decimal formatter: {}", e))?;

        Ok(Self { locale, decimal_formatter, display_decimals: None })
    }

    pub fn with_display_decimals(mut self, decimals: u8) -> Self {
        self.display_decimals = Some(decimals);
        self
    }

    fn adjust_precision(
        decimal: &mut FixedDecimal,
        target_decimals: i16,
        currency_decimals: i16,
    ) {
        println!("XDB - adjust_precision: initial decimal={}", decimal);
        println!(
            "XDB - adjust_precision: target_decimals={}, currency_decimals={}",
            target_decimals, currency_decimals
        );

        decimal.multiply_pow10(-currency_decimals);
        println!("XDB - adjust_precision: after minor to major={}", decimal);

        let round_position = -target_decimals;
        decimal.half_even(round_position);
        println!(
            "XDB - adjust_precision: after half_even at {}={}",
            round_position, decimal
        );

        decimal.pad_end(target_decimals);
        println!("XDB - adjust_precision: after pad_end={}", decimal);
    }

    pub fn format(&self, money: &Money) -> String {
        println!(
            "XDB - format: starting with minor_value={}, symbol={}",
            money.minor_value(),
            money.symbol()
        );

        let minor_value = money.minor_value().to_string();
        let mut decimal =
            FixedDecimal::from_str(&minor_value).expect("Valid decimal");
        println!("XDB - format: initial decimal={}", decimal);

        let currency_decimals = money.decimals() as i16;
        println!("XDB - format: currency_decimals={}", currency_decimals);

        if let Some(display_decimals) = self.display_decimals {
            println!(
                "XDB - format: applying precision with display_decimals={}",
                display_decimals
            );
            Self::adjust_precision(
                &mut decimal,
                display_decimals as i16,
                currency_decimals,
            );
        } else {
            println!("XDB - format: no precision specified, converting to major units");
            decimal.multiply_pow10(-currency_decimals);
        }
        println!("XDB - format: final decimal before formatting={}", decimal);

        let formatted = self.decimal_formatter.format(&decimal).to_string();
        println!("XDB - format: ICU formatted={}", formatted);

        let result = format!("{} {}", formatted, money.symbol());
        println!("XDB - format: final result={}", result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Currency, Money};
    use alloy::primitives::U256;

    #[test]
    fn test_eth_formatting() {
        {
            let formatter = MoneyFormatter::default().with_display_decimals(6);

            let one_eth = Money::eth(U256::from(1_000_000_000_000_000_000u128));
            assert_eq!(formatter.format(&one_eth), "1.000000 ETH");

            let half_eth = Money::eth(U256::from(500_000_000_000_000_000u128));
            assert_eq!(formatter.format(&half_eth), "0.500000 ETH");

            let one_gwei = Money::eth(U256::from(1_000_000_000u128));
            assert_eq!(formatter.format(&one_gwei), "0.000000 ETH");

            let zero_eth = Money::zero_eth();
            assert_eq!(formatter.format(&zero_eth), "0.000000 ETH");
        }

        {
            let formatter = MoneyFormatter::default();

            let one_eth = Money::eth(U256::from(1_000_000_000_000_000_000u128));
            assert_eq!(formatter.format(&one_eth), "1.000000000000000000 ETH");

            let half_eth = Money::eth(U256::from(500_000_000_000_000_000u128));
            assert_eq!(formatter.format(&half_eth), "0.500000000000000000 ETH");

            let one_gwei = Money::eth(U256::from(1_000_000_000u128));
            assert_eq!(formatter.format(&one_gwei), "0.000000001000000000 ETH");

            let zero_eth = Money::zero_eth();
            assert_eq!(formatter.format(&zero_eth), "0.000000000000000000 ETH");
        }
    }

    #[test]
    fn test_high_precision_usd() {
        let formatter = MoneyFormatter::default().with_display_decimals(2);

        let usd =
            Money::new(U256::from(123456789123u128), Currency::usd(), Some(4));
        assert_eq!(formatter.format(&usd), "123,456.79 USD");

        let usd_even =
            Money::new(U256::from(123456785000u128), Currency::usd(), Some(4));
        assert_eq!(formatter.format(&usd_even), "123,456.78 USD");

        let usd_odd =
            Money::new(U256::from(123456775000u128), Currency::usd(), Some(4));
        assert_eq!(formatter.format(&usd_odd), "123,456.78 USD");
    }

    #[test]
    fn test_different_locales() {
        let us = MoneyFormatter::default();
        let swiss = MoneyFormatter::new("fr-CH").unwrap();
        let uk = MoneyFormatter::new("en-GB").unwrap();

        let money = Money::new(U256::from(123456u128), Currency::usd(), None);

        let us_formatted = us.format(&money);
        let swiss_formatted = swiss.format(&money);
        let uk_formatted = uk.format(&money);

        assert_eq!(us_formatted, "1,234.56 USD");
        assert_eq!(swiss_formatted, "1\u{202f}234,56 USD");
        assert_eq!(uk_formatted, "1,234.56 USD");
    }

    #[test]
    fn test_different_currencies() {
        let formatter = MoneyFormatter::default();

        let jpy = Money::new(U256::from(1234u128), Currency::jpy(), None);
        assert_eq!(formatter.format(&jpy), "1,234 JPY");

        let mga = Money::new(U256::from(12345u128), Currency::mga(), None);
        assert_eq!(formatter.format(&mga), "1,234.5 MGA");

        let usd = Money::new(U256::from(123456u128), Currency::usd(), None);
        assert_eq!(formatter.format(&usd), "1,234.56 USD");

        let bhd = Money::new(U256::from(1234567u128), Currency::bhd(), None);
        assert_eq!(formatter.format(&bhd), "1,234.567 BHD");
    }

    #[test]
    fn test_currency_decimals() {
        assert_eq!(Currency::jpy().decimals(), 0);
        assert_eq!(Currency::mga().decimals(), 1);
        assert_eq!(Currency::usd().decimals(), 2);
        assert_eq!(Currency::bhd().decimals(), 3);
    }
}
