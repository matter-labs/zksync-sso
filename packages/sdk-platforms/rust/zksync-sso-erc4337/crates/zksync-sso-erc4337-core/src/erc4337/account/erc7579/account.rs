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
    struct Execution {
        address target;
        uint256 value;
        bytes data;
    }

    function execute(bytes32 mode, bytes memory execution);
}
