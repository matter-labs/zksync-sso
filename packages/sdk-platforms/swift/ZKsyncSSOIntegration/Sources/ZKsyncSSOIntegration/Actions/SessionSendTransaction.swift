import Foundation
import ZKsyncSSO

public func sessionSendTransaction(
    transaction: TransactionRequest = TransactionRequest(
        to: IntegrationConstants.sessionSendTestTransferTarget,
        value: IntegrationConstants.sessionSendTestTransferAmount,
        input: nil
    )
) async throws {
    print("🚀 Running session send transaction...")

    print("\n=== STEP 1: Deploy Account with Initial Session ===")
    
    // Deploy the account using the extracted function
    let deployedAccount = try await deployAccountForSessionSend()
    let deployedAccountAddress = deployedAccount.address
    let sessionConfigJson = deployedAccount.sessionConfigJson
    let config = deployedAccount.config
    
    guard let sessionConfigJson = sessionConfigJson else {
        fatalError("Missing session config JSON")
    }

    print("\n=== STEP 2: Verify Initial Session State ===")

    let expectedFeeLimit = "100000000000000000"  // 0.1 ETH
    let initialSessionState = try await validateSessionState(
        accountAddress: deployedAccountAddress,
        sessionConfig: sessionConfigJson,
        predicates: [
            sessionStateActive(),
            sessionStateFeesRemainingEquals(expectedFeeLimit)
        ],
        config: config,
        description: "Initial session state after deployment"
    )

    print("\n=== STEP 3: Fund the Deployed Account ===")

    print("💰 Funding account \(deployedAccountAddress) with 1 ETH...")
    try await ZKsyncSSO.fundAccount(
        address: deployedAccountAddress,
        amount: "1.0",
        config: config
    )
    print("✅ Account funded successfully!")

    print("\n=== STEP 4: Send Transaction Using Session ===")

    // Verify session key bytes match expected value (matching Rust test validation)
    let sessionKeyHex = IntegrationConstants.sessionSendTestSessionOwnerPrivateKey
    let expectedSessionKeyBytes = IntegrationConstants.sessionSendTestExpectedSessionKeyBytes
    
    try validateSessionKey(sessionKeyHex: sessionKeyHex, expectedBytes: expectedSessionKeyBytes)
    
    // Send the transaction using the session
    _ = try await sessionSend(
        deployedAccount: deployedAccount,
        sessionKey: sessionKeyHex,
        sessionConfig: sessionConfigJson,
        transaction: transaction,
    )

    print("\n=== STEP 5: Verify Session State After Transaction ===")

    let sessionStateAfterTx = try await validateSessionState(
        accountAddress: deployedAccountAddress,
        sessionConfig: sessionConfigJson,
        predicates: [
            sessionStateActive(),
            sessionStateFeesConsumed(from: initialSessionState.feesRemaining)
        ],
        config: config,
        description: "Session state after transaction"
    )

    // Log fees consumed for debugging
    _ = try validateFeesConsumed(
        initialFees: initialSessionState.feesRemaining,
        remainingFees: sessionStateAfterTx.feesRemaining,
        description: "Transaction fees"
    )

    // Validate transfer values if they exist
    if !sessionStateAfterTx.transferValue.isEmpty {
        try validateSessionStateImpl(
            actual: sessionStateAfterTx,
            predicates: [
                sessionStateCustom("transfer value[0] should have remaining amount > 0") { state in
                    !state.transferValue.isEmpty && state.transferValue[0].remaining != "0"
                }
            ],
            description: "Transfer value validation"
        )
        
        let transferValueRemaining = sessionStateAfterTx.transferValue[0].remaining
        print("💸 Transfer value remaining: \(transferValueRemaining)")
    }

    print("\n" + String(repeating: "=", count: 80))
    print("✅ SESSION SEND COMPLETED SUCCESSFULLY ✅")
    print(String(repeating: "=", count: 80))
    print("Summary:")
    print("1. Deployed smart account with initial session")
    print("2. Funded the account with 1 ETH")
    print("3. Sent 0.005 ETH using session key")
    print("4. Verified session remains active with updated state")
}
