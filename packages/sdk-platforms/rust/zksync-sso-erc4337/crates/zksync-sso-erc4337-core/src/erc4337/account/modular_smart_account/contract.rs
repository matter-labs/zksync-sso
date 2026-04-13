use alloy::sol;

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    MSAFactory,
    "../../../../../../packages/contracts/out/MSAFactory.sol/MSAFactory.json"
);

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    ModularSmartAccount,
    "../../../../../../packages/contracts/out/ModularSmartAccount.sol/ModularSmartAccount.json"
);
