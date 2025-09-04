import SwiftUI
import ZKsyncSSO
import ZKsyncSSOIntegration
import AuthenticationServices

struct TransactionConfigurationFactory {
    static func regularTransaction(
        fromAccount: DeployedAccount,
        authorizationController: AuthorizationController
    ) -> TransactionConfiguration {
        TransactionConfiguration(
            title: "Send Transaction",
            buttonTitle: "Send Transaction",
            buttonProgressTitle: "Sending Transaction...",
            initialToAddress: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            initialAmount: "1.0",
            prepareTransaction: { toAddress, amount in
                try await prepareRegularTransaction(
                    fromAccount: fromAccount,
                    toAddress: toAddress,
                    amount: amount,
                    authorizationController: authorizationController
                )
            },
            confirmTransaction: { toAddress, amount in
                try await confirmRegularTransaction(
                    fromAccount: fromAccount,
                    toAddress: toAddress,
                    amount: amount,
                    authorizationController: authorizationController
                )
            }
        )
    }
    
    static func sessionTransaction(
        session: Session,
        account: DeployedAccount
    ) -> TransactionConfiguration {
        TransactionConfiguration(
            title: "Send Session Transaction",
            buttonTitle: "Send Transaction",
            buttonProgressTitle: "Sending Transaction...",
            initialToAddress: "0xdebbd4ce2bd6bd869d3ac93666a0d5f4fc06fc72",
            initialAmount: "1.0",
            prepareTransaction: { toAddress, amount in
                try await prepareSessionTransaction(
                    toAddress: toAddress,
                    amount: amount,
                    account: account
                )
            },
            confirmTransaction: { toAddress, amount in
                try await confirmSessionTransaction(
                    session: session,
                    account: account,
                    toAddress: toAddress,
                    amount: amount
                )
            }
        )
    }
    
    // MARK: - Regular Transaction Helpers
    
    private static func prepareRegularTransaction(
        fromAccount: DeployedAccount,
        toAddress: String,
        amount: String,
        authorizationController: AuthorizationController
    ) async throws -> PreparedTransaction {
        guard let amountInEth = Double(amount) else {
            throw NSError(domain: "InvalidAmount", code: 1, userInfo: [NSLocalizedDescriptionKey: "Invalid amount"])
        }
        
        let amountInWei = String(Int(amountInEth * 1_000_000_000_000_000_000.0))
        
        let authenticator = await PasskeyAuthenticatorHelper(
            controllerProvider: { authorizationController },
            relyingPartyIdentifier: "auth-test.zksync.dev"
        )
        
        let accountClient = AccountClient(
            account: .init(
                address: fromAccount.address,
                uniqueAccountId: fromAccount.uniqueAccountId
            ),
            authenticatorAsync: PasskeyAuthAsync(
                authenticator: authenticator
            )
        )
        
        let transaction = TransactionRequest(
            to: toAddress,
            value: amountInWei
        )
        
        return try await accountClient.prepare(transaction: transaction)
    }
    
    private static func confirmRegularTransaction(
        fromAccount: DeployedAccount,
        toAddress: String,
        amount: String,
        authorizationController: AuthorizationController
    ) async throws {
        guard let amountInEth = Double(amount) else {
            throw NSError(domain: "InvalidAmount", code: 1, userInfo: [NSLocalizedDescriptionKey: "Invalid amount"])
        }
        
        let amountInWei = String(Int(amountInEth * 1_000_000_000_000_000_000.0))
        
        let authenticator = await PasskeyAuthenticatorHelper(
            controllerProvider: { authorizationController },
            relyingPartyIdentifier: "auth-test.zksync.dev"
        )
        
        let accountClient = AccountClient(
            account: .init(
                address: fromAccount.address,
                uniqueAccountId: fromAccount.uniqueAccountId
            ),
            authenticatorAsync: PasskeyAuthAsync(
                authenticator: authenticator
            )
        )
        
        try await accountClient.send(
            transaction: .init(
                to: toAddress,
                value: amountInWei
            )
        )
    }
    
    // MARK: - Session Transaction Helpers
    
    private static func prepareSessionTransaction(
        toAddress: String,
        amount: String,
        account: DeployedAccount
    ) async throws -> PreparedTransaction {
        guard let amountInEth = Double(amount) else {
            throw NSError(domain: "InvalidAmount", code: 1, userInfo: [NSLocalizedDescriptionKey: "Invalid amount"])
        }
        
        let amountInWei = String(Int(amountInEth * 1_000_000_000_000_000_000.0))
        
        // For session transactions, we typically don't prepare them the same way
        // Return a placeholder PreparedTransaction since SessionClient handles this internally
        return PreparedTransaction(
            transactionRequestJson: "{}",
            from: account.address,
            to: toAddress,
            value: amountInWei,
            displayFee: "Session Transaction Fee"
        )
    }
    
    private static func confirmSessionTransaction(
        session: Session,
        account: DeployedAccount,
        toAddress: String,
        amount: String
    ) async throws {
        // Parameters are kept for future use, but currently ignored
        // Using ZKsyncSSOIntegration.sessionSendTransaction which has its own test parameters
        print("🚀 Using ZKsyncSSOIntegration.sessionSendTransaction for transaction...")
        print("  Note: Currently using test parameters from integration, ignoring provided params")
        print("  Provided params (for future use):")
        print("    - Session: \(session.sessionHash)")
        print("    - Account: \(account.address)")
        print("    - To: \(toAddress)")
        print("    - Amount: \(amount)")
        
        // Call the integration function which handles the complete session transaction flow
        try await ZKsyncSSOIntegration.sessionSendTransaction()
        
        print("✅ Session transaction completed successfully!")
    }
}
