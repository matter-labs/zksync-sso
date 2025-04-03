use alloy::{network::TransactionBuilder, primitives::U256};
use alloy_zksync::network::transaction_request::TransactionRequest;
use money::{Money, MoneyFormatter};

pub fn calculate_display_fee(tx: &TransactionRequest) -> String {
    let gas_limit = tx.gas_limit().unwrap_or_default();
    let max_fee_per_gas = tx.max_fee_per_gas().unwrap_or_default();

    let total_fee = U256::from(gas_limit) * U256::from(max_fee_per_gas);

    let money = Money::eth(total_fee);

    let formatter = MoneyFormatter::default().with_display_decimals(6);
    formatter.format(&money)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_display_fee() {
        let mut tx = TransactionRequest::default();
        tx.set_gas_limit(100_000u64);
        tx.set_max_fee_per_gas(50_000_000_000u128);

        let display_fee = calculate_display_fee(&tx);
        assert_eq!(display_fee, "0.005000 ETH");

        let tx_zero = TransactionRequest::default();
        assert_eq!(calculate_display_fee(&tx_zero), "0.000000 ETH");

        let mut tx_max = TransactionRequest::default();
        tx_max.set_gas_limit(u64::MAX);
        tx_max.set_max_fee_per_gas(u128::MAX);
        let max_fee = calculate_display_fee(&tx_max);

        let expected =
            U256::from(u64::MAX).saturating_mul(U256::from(u128::MAX));
        let expected_money = Money::eth(expected);
        let expected_formatted = MoneyFormatter::default()
            .with_display_decimals(6)
            .format(&expected_money);
        assert_eq!(max_fee, expected_formatted);
    }
}
