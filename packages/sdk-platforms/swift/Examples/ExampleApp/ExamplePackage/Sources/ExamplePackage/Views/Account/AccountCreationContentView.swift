import AuthenticationServices
import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO
import ZKsyncSSOIntegration

struct AccountCreationContentView: View {
    typealias OnSuccess = () -> Void

    @Environment(\.dismiss) private var dismiss

    @Environment(\.authorizationController) private var authorizationController

    @EnvironmentObject private var sessionsStore: SessionsStore

    @State private var isLoading = false
    @State private var error: Error?
    @State private var deployedAddress: String?

    private let accountInfo: AccountInfo
    private let challenge: Data
    private let signers: AccountSigners

    let onSuccess: OnSuccess
    let onDeployed: OnDeployed

    init(
        challenge: Data,
        accountInfo: AccountInfo,
        signers: AccountSigners = .default,
        onSuccess: @escaping () -> Void,
        onDeployed: @escaping OnDeployed
    ) {
        self.challenge = challenge
        self.accountInfo = accountInfo
        self.signers = signers
        self.onSuccess = onSuccess
        self.onDeployed = onDeployed
    }

    var body: some View {
        VStack(spacing: 16) {
            Button {
                createPasskey()
            } label: {
                HStack {
                    Image(systemName: "key.fill")
                        .font(.system(size: 16))
                    Text("Create Passkey")
                        .font(.headline)
                }
                .frame(maxWidth: .infinity)
                .frame(height: 50)
                .background(Color.blue)
                .foregroundColor(.white)
                .cornerRadius(10)
            }
            .disabled(isLoading)
            
            if ExampleConfiguration.showSessionAccountFlowView {
                Button {
                    createAccountWithOwner()
                } label: {
                    HStack {
                        Image(systemName: "person.circle.fill")
                            .font(.system(size: 16))
                        Text("Create Account with K1 Owner")
                            .font(.headline)
                    }
                    .frame(maxWidth: .infinity)
                    .frame(height: 50)
                    .background(Color.green)
                    .foregroundColor(.white)
                    .cornerRadius(10)
                }
                .disabled(isLoading)
            }
            
            if isLoading {
                ProgressView()
                    .padding(.top, 8)
            }

            if let error = error {
                Text(error.localizedDescription)
                    .foregroundStyle(.red)
                    .font(.footnote)
                    .padding(.top, 8)
            }
        }
        .navigationDestination(for: DeployedAccount.self) { deployedAccount in
            AccountDetailsView(account: .init(account: deployedAccount))
        }
        .id("PasskeyCreationView")
        .onAppear { print("PasskeyCreationView appeared") }
    }

    @MainActor
    private func createPasskey() {
        isLoading = true
        error = nil

        Task { @MainActor in
            do {
                let initialK1Owners: [String]? = [signers.accountOwner.address]
                // Use debug constants for session owner to match deployment
                let sessionKey = signers.sessionOwner.privateKeyHex
                let initialSessionConfig = SessionSpec(
                    signer: IntegrationConstants.sessionOwner.address,
                    expiresAt: String(IntegrationConstants.expiresAt),
                    feeLimit: UsageLimit(
                        limitType: .lifetime,
                        limit: IntegrationConstants.feeLimitLifetime,
                        period: "0"
                    ),
                    callPolicies: [],
                    transferPolicies: [
                        TransferSpec(
                            target: IntegrationConstants.transferSessionTarget,
                            maxValuePerUse: IntegrationConstants.maxValuePerUse,
                            valueLimit: UsageLimit(
                                limitType: .unlimited,
                                limit: "0",
                                period: "0"
                            )
                        )
                    ]
                )
                let initialSessionConfigJson = try initialSessionConfig.toJsonString()

                let account = try await createAccount(
                    userName: accountInfo.name,
                    userID: accountInfo.userID,
                    challenge: challenge,
                    relyingPartyIdentifier: accountInfo.domain,
                    initialK1Owners: initialK1Owners,
                    initialSessionConfigJson: nil,
                    controller: authorizationController
                )
                let address = account.address
                let uniqueAccountId = account.uniqueAccountId
                print("🏗️  Account deployed at address: \(address)")
                print("   with uniqueAccountId: \(uniqueAccountId)")
                print("   Expected owner address: \(signers.accountOwner.address)")
                print("   Expected owner private key: \(signers.accountOwner.privateKeyHex)")

                // Validate that the account owner is indeed a K1 owner
                print("Validating K1 owner...")
                print("Account address: \(address)")
                print("Owner address being validated: \(signers.accountOwner.address)")
                print("Owner private key: \(signers.accountOwner.privateKeyHex)")
                print("Initial K1 owners used in deployment: \(initialK1Owners ?? [])")

                let isK1OwnerArgs = IsK1OwnerArgs(
                    account: address,
                    ownerAddress: signers.accountOwner.address
                )
                let isValidK1Owner = try await isK1Owner(
                    args: isK1OwnerArgs,
                    config: Config.default
                )
                print("K1 owner validation result: \(isValidK1Owner)")

                if !isValidK1Owner {
                    throw UIError(message: "Account owner validation failed - not a valid K1 owner")
                }

                // Fund the account with 1 ETH to match debug actions
                print("Funding account with 1 ETH...")
                try await fundAccount(
                    address: address,
                    amount: "0.5",
                    config: Config.default
                )  // 1 ETH in wei
                print("Account funded successfully")

                let deployedAccount = DeployedAccount(
                    info: accountInfo,
                    address: address,
                    uniqueAccountId: uniqueAccountId
                )

                print("Deployed account: \(deployedAccount)")

                let initialSession = Session.create(
                    sessionKey: sessionKey,
                    sessionSpec: initialSessionConfig
                )
                sessionsStore.addSession(initialSession, for: address)

                // Signal success to parent view
                onSuccess()

                // Provide deployed account and signer to callback
                onDeployed(deployedAccount, signers)
            } catch let error as ASAuthorizationError where error.code == .canceled {
                print("User cancelled passkey creation")
            } catch {
                print("error: \(error)")
                self.error = error
            }
            isLoading = false
        }
    }

    @MainActor
    private func createAccountWithOwner() {
        isLoading = true
        error = nil

        Task { @MainActor in
            do {
                print("🚀 Creating account with K1 Owner using ZKsyncSSOIntegration.deployAccount...")
                let sessionOwner = signers.sessionOwner
                let sessionKey = sessionOwner.privateKeyHex
                
                let deployedAccountDetails = try await ZKsyncSSOIntegration.deployAccount()
                let uniqueAccountId = deployedAccountDetails.uniqueAccountId
                let initialSessionConfig = deployedAccountDetails.sessionConfigJson
                
                print("✅ Account deployment completed!")
                print("   Address: \(deployedAccountDetails.address)")
                print("   Unique ID: \(String(describing: uniqueAccountId))")

                let deployedAccount = DeployedAccount(
                    info: accountInfo,
                    address: deployedAccountDetails.address,
                    uniqueAccountId: uniqueAccountId!
                )

                print("Deployed account: \(deployedAccount)")

                let initialSession = Session.create(
                    sessionKey: sessionKey,
                    sessionSpec: try SessionSpec.fromJsonString(initialSessionConfig!)
                )
                sessionsStore.addSession(initialSession, for: deployedAccount.address)

                // Signal success to parent view
                onSuccess()

                // Provide deployed account and signer to callback
                onDeployed(deployedAccount, signers)
            } catch {
                print("error: \(error)")
                self.error = error
            }
            isLoading = false
        }
    }
}

#Preview {
    AccountCreationContentView(
        challenge: Data("example-challenge".utf8),
        accountInfo: .init(
            name: "Jane Doe",
            userID: "jdoe",
            domain: "auth-test.zksync.dev"
        ),
        onSuccess: {},
        onDeployed: { _, _ in }
    )
}
