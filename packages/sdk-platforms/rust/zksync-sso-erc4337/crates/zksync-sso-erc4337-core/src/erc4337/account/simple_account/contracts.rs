use alloy::sol;

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
