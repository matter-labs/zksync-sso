use alloy::primitives::{Address, Bytes, U256};

#[derive(Debug, Clone)]
pub struct Call {
    pub to: Address,
    pub value: U256,
    pub data: Bytes,
}

#[derive(Debug, Clone)]
pub struct SendCalls {
    pub account: Address,
    pub calls: Vec<Call>,
}
