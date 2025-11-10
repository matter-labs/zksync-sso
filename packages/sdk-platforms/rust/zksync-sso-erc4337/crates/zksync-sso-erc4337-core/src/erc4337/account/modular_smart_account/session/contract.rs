use alloy::sol;

sol!(
    #[sol(rpc)]
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    SessionKeyValidator,
    "../../../../../../packages/erc4337-contracts/out/SessionKeyValidator.sol/SessionKeyValidator.json"
);
