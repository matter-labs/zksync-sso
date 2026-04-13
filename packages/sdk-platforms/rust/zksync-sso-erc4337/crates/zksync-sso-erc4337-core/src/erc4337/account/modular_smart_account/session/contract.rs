use alloy::sol;

sol!(
    #[sol(rpc)]
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    SessionKeyValidator,
    "../../../../../../packages/contracts/out/SessionKeyValidator.sol/SessionKeyValidator.json"
);
