import AuthenticationServices
import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

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
        VStack {
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
                let initialK1Owners: [String]? = [ signers.accountOwner.address ]
                let initialSessionConfig = SessionSpec.initialSession(
                    sessionOwner: signers.sessionOwner
                )
                let initialSessionConfigJson = try initialSessionConfig.toJsonString()

                let account = try await createAccount(
                    userName: accountInfo.name,
                    userID: accountInfo.userID,
                    challenge: challenge,
                    relyingPartyIdentifier: accountInfo.domain,
                    initialK1Owners: initialK1Owners,
                    initialSessionConfigJson: initialSessionConfigJson,
                    controller: authorizationController
                )
                let address = account.address
                let uniqueAccountId = account.uniqueAccountId
                print("Account deployed at address: \(address)")
                print("with uniqueAccountId: \(uniqueAccountId)")
                let deployedAccount = DeployedAccount(
                    info: accountInfo,
                    address: address,
                    uniqueAccountId: uniqueAccountId
                )

                print("Deployed account: \(deployedAccount)")

                let initialSession = Session(
                    createdAt: Date(),
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
