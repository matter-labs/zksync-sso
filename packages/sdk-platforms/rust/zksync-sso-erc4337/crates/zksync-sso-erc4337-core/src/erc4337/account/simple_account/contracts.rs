use alloy::sol;

// SimpleAccount + Factory artifacts (from dependency compiled via vendor shim)
sol!(
    #[sol(rpc)]
    SimpleAccountFactory,
    "../../../../../../packages/erc4337-contracts/out/SimpleAccountFactory.sol/SimpleAccountFactory.json"
);

sol!(
    #[sol(rpc)]
    SimpleAccount,
    "../../../../../../packages/erc4337-contracts/out/SimpleAccount.sol/SimpleAccount.json"
);
