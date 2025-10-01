pub mod add_module;
pub mod calls;
pub mod module_installed;

use alloy::sol;

sol!(
    #[sol(rpc)]
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    IERC7579Account,
    "../../../../../../packages/erc4337-contracts/out/IERC7579Account.sol/IERC7579Account.json"
);

sol! {
    #[sol(rpc)]
    #[derive(Debug, Default)]
    struct Execution {
        address target;
        uint256 value;
        bytes data;
    }

    function execute(bytes32 mode, bytes memory execution);
}
