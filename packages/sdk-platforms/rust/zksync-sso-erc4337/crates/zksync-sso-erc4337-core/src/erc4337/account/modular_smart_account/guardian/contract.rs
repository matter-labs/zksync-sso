use alloy::sol;

sol!(
    #[sol(rpc)]
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    GuardianExecutor,
    "../../../../../../packages/erc4337-contracts/out/GuardianExecutor.sol/GuardianExecutor.json"
);
