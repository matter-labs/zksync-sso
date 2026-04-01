pub mod entry_point;
use alloy::sol;

sol!(
    #[sol(rpc)]
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    AccountBase,
    "../../../../../../packages/contracts/out/AccountBase.sol/AccountBase.json"
);
