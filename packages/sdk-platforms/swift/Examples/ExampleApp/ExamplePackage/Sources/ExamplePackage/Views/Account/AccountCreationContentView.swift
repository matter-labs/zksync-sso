import AuthenticationServices
import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

struct AccountCreationContentView: View {

    typealias OnDeployed = (DeployedAccount) -> Void
    typealias OnSuccess = () -> Void

    @Environment(\.dismiss) private var dismiss

    @Environment(\.authorizationController) private var authorizationController

    @State private var isLoading = false
    @State private var error: Error?
    @State private var deployedAddress: String?

    private let accountInfo: AccountInfo
    private let challenge: Data

    let onSuccess: OnSuccess
    let onDeployed: OnDeployed

    init(
        challenge: Data,
        accountInfo: AccountInfo,
        onSuccess: @escaping () -> Void,
        onDeployed: @escaping (DeployedAccount) -> Void
    ) {
        self.challenge = challenge
        self.accountInfo = accountInfo
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
                let secretAccountSalt = Data(repeating: 0, count: 32)
                let account = try await createAccount(
                    userName: accountInfo.name,
                    userID: accountInfo.userID,
                    secretAccountSalt: secretAccountSalt,
                    challenge: challenge,
                    relyingPartyIdentifier: accountInfo.domain,
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

                print("XXX deployed account: \(deployedAccount)")

                // Signal success to parent view
                onSuccess()
                
                // Provide deployed account to callback
                onDeployed(deployedAccount)

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
            domain: "soo-sdk-example-pages.pages.dev"
        ),
        onSuccess: {},
        onDeployed: { _ in }
    )
}
