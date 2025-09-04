import Foundation
import ZKsyncSSO

public func sessionSend(
    deployedAccount: DeployedAccountDetails,
    sessionKey: String,
    sessionConfig: String,
    transaction: TransactionRequest,
) async throws -> String {
    print("\n=== Sending Transaction Using Session ===")
    
    let accountAddress = deployedAccount.address
    let config = deployedAccount.config
    
    // Create SessionClient
    let sessionClient = SessionClient(
        accountAddress: accountAddress,
        sessionKey: sessionKey,
        sessionConfig: sessionConfig,
        config: config
    )
    
    print("Sending transaction...")
    print("  From: \(accountAddress)")
    print("  To: \(transaction.to ?? "nil")")
    print("  Value: \(transaction.value ?? "0") wei")
    if let input = transaction.input {
        print("  Input: \(input)")
    }
    
    print("🔄 Executing session send transaction...")
    print("  This internally will:")
    print("  1. Set from field to account address")
    print("  2. Populate transaction request with chain data")
    print("  3. Create session management transaction hash")
    print("  4. Sign hash using session key")
    print("  5. Build raw transaction")
    print("  6. Send via provider")
    
    // Send the transaction
    let sessionTransactionReceipt = try await sessionClient.sendTransaction(
        transaction: transaction
    )
    
    print("✅ Transaction confirmed successfully!")
    print("  Transaction receipt: \(sessionTransactionReceipt)")
    
    return sessionTransactionReceipt
}

// Note: validateSessionKey function has been moved to Utils.swift
