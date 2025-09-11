use alloy::{
    primitives::Address,
    sol_types::{Eip712Domain, eip712_domain},
};

pub fn build_domain(entry_point: &Address, chain_id: u64) -> Eip712Domain {
    eip712_domain! {
        name: "ERC4337",
        version: "1",
        chain_id: chain_id,
        verifying_contract: *entry_point,
    }
}
