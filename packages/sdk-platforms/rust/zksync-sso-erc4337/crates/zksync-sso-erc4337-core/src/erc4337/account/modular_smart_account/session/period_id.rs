use crate::erc4337::account::modular_smart_account::{
    session::{
        session_lib::session_spec::{
            limit_type::LimitType, usage_limit::UsageLimit,
        },
    },
};
use alloy::primitives::Uint;

pub fn get_period_id(limit: &UsageLimit) -> Uint<48, 1> {
    let current_timestamp = Uint::<48, 1>::from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

    if limit.limit_type == LimitType::Allowance {
        current_timestamp / limit.period
    } else {
        Uint::from(0)
    }
}
