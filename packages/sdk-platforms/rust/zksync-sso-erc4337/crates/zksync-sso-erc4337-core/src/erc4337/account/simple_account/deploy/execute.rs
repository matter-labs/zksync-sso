use crate::erc4337::account::simple_account::contracts::SimpleAccount::executeCall;
use alloy::{
    primitives::{Address, Bytes, U256},
    sol_types::SolCall,
};

pub struct SimpleAccountExecute(executeCall);

impl SimpleAccountExecute {
    pub fn new(target: Address, value: U256, data: Bytes) -> Self {
        Self(executeCall { target, value, data })
    }

    pub fn encode(&self) -> Vec<u8> {
        executeCall::abi_encode(&self.0)
    }
}
