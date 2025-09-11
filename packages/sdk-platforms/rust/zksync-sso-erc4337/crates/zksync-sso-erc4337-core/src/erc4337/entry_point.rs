use alloy::sol;

pub mod config;
pub mod nonce;
pub mod sender_address;
pub mod version;

sol!(
    #[sol(rpc)]
    EntryPoint,
    "../../../../../../packages/erc4337-contracts/out/EntryPoint.sol/EntryPoint.json"
);

pub use self::EntryPoint::PackedUserOperation;
