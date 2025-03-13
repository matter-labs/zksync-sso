use crate::{config::Config, utils::alloy::conveniences::get_provider};
use alloy::{primitives::Address, providers::Provider};
use money::{Money, MoneyFormatter};

#[derive(Debug)]
pub struct GetAccountBalanceResult {
    pub balance: String,
}

pub async fn get_balance(
    address: Address,
    config: &Config,
) -> eyre::Result<GetAccountBalanceResult> {
    println!("XDB get_balance - address: {:?}", address);

    let provider = get_provider(config)?;
    let balance_uint = provider.get_balance(address).await?;

    let money = Money::eth(balance_uint);
    
    let formatter = MoneyFormatter::default().with_display_decimals(6);
    let balance = formatter.format(&money);

    let balance = GetAccountBalanceResult { balance };
    println!("XDB get_balance - balance: {:?}", balance);

    Ok(balance)
}
