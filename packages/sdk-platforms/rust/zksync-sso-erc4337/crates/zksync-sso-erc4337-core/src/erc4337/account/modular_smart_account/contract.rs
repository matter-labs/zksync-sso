use alloy::sol;

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    MSAFactory,
    "../../../../../../packages/erc4337-contracts/out/MSAFactory.sol/MSAFactory.json"
);

sol!(
    #[derive(Debug, Default)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    ModularSmartAccount,
    "../../../../../../packages/erc4337-contracts/out/ModularSmartAccount.sol/ModularSmartAccount.json"
);
