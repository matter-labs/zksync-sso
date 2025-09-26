use alloy::sol;

// Import contract ABIs and bytecode from compiled artifacts
// Note: These contracts define their own types including PackedUserOperation
sol!(
    #[sol(rpc)]
    EntryPoint,
    "../../../../../contracts/out/EntryPoint.sol/EntryPoint.json"
);

sol!(
    #[sol(rpc)]
    SimpleAccountFactory,
    "../../../../../contracts/out/SimpleAccountFactory.sol/SimpleAccountFactory.json"
);

sol!(
    #[sol(rpc)]
    SimpleAccount,
    "../../../../../contracts/out/SimpleAccount.sol/SimpleAccount.json"
);

// Re-export types for easier access
// pub use self::EntryPoint::{IEntryPoint, IStakeManager};
// Re-export PackedUserOperation from the generated contract types
pub use self::EntryPoint::PackedUserOperation;
