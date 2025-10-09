pub mod create;
pub mod hash;
pub mod revoke;
pub mod send;
pub mod status;

use alloy::sol;

sol!(
    #[sol(rpc)]
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    SessionKeyValidator,
    "../../../../../../packages/erc4337-contracts/out/SessionKeyValidator.sol/SessionKeyValidator.json"
);

mod session_lib {
    use alloy::sol;

    sol!(
        #[sol(rpc)]
        #[derive(Debug, Default)]
        #[allow(missing_docs)]
        SessionLib,
        "../../../../../../packages/erc4337-contracts/out/SessionLib.sol/SessionLib.json"
    );
}
