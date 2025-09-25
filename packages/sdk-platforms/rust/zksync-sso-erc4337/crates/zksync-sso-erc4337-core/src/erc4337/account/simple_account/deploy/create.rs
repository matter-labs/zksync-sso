use crate::erc4337::account::simple_account::deploy::SimpleAccountFactory::createAccountCall;
use alloy::{
    primitives::{Address, U256},
    sol_types::SolCall,
};

pub struct SimpleAccountCreate(createAccountCall);

impl SimpleAccountCreate {
    pub fn new(owner: Address, salt: U256) -> Self {
        Self(createAccountCall { owner, salt })
    }

    pub fn new_u64(owner: Address, salt: u64) -> Self {
        let salt = U256::from(salt);
        Self(createAccountCall { owner, salt })
    }

    pub fn encode(&self) -> Vec<u8> {
        createAccountCall::abi_encode(&self.0)
    }
}
