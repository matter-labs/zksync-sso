pub mod create;
pub mod hash;
pub mod revoke;
pub mod send;
pub mod session_lib;
pub mod status;

use alloy::sol;

sol!(
    #[sol(rpc)]
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    SessionKeyValidator,
    "../../../../../../packages/erc4337-contracts/out/SessionKeyValidator.sol/SessionKeyValidator.json"
);
