import AuthenticationServices
import ExamplePackageUIComponents
import SwiftUI

struct PasskeyCreationView: View {

    typealias OnDeployed = (DeployedAccount) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var accountStore = AccountStore()
    @State private var accountCreator: PasskeyAccountCreator
    @State private var isLoading = false
    @State private var error: Error?
    @State private var deployedAddress: String?
    @State private var showingSuccess = false

    private let accountInfo: AccountInfo
    private let challenge: Data
    private let passkeyManager: PasskeyManager
    let onDeployed: OnDeployed

    init(
        challenge: Data,
        accountInfo: AccountInfo,
        onDeployed: @escaping (DeployedAccount) -> Void
    ) {
        self.challenge = challenge
        self.accountInfo = accountInfo
        self.onDeployed = onDeployed
        self.passkeyManager = PasskeyManager(
            relyingPartyIdentifier: accountInfo.domain
        )
        self.accountCreator = PasskeyAccountCreator(
            manager: passkeyManager
        )
    }

    var body: some View {
        NavigationStack {
            VStack {
                ActionButton(
                    title: "Create Passkey",
                    progressTitle: "Creating Passkey...",
                    icon: "key.fill",
                    isLoading: isLoading,
                    style: .prominent,
                    action: createPasskey
                )
                .padding(.horizontal)

                if let error = error {
                    Text(error.localizedDescription)
                        .foregroundStyle(.red)
                }
            }
            .navigationDestination(for: DeployedAccount.self) { deployedAccount in
                AccountDetailsView(
                    address: deployedAccount.address,
                    uniqueAccountId: deployedAccount.uniqueAccountId,
                    relyingPartyIdentifier: accountInfo.domain
                )
            }
            .overlay {
                if showingSuccess {
                    ToastView(
                        icon: "checkmark.circle.fill",
                        iconColor: .green,
                        message: "Account Deployed!"
                    )
                }
            }
        }
        .passkeyPresentation(passkeyManager)
        .id("PasskeyCreationView")
        .onAppear { print("PasskeyCreationView appeared") }
    }

    @MainActor
    private func createPasskey() {
        isLoading = true
        error = nil

        Task { @MainActor in
            do {
                let account = try await accountCreator.createAccount(
                    userName: accountInfo.name,
                    userID: accountInfo.userID,
                    challenge: challenge
                )
                let address = account.address
                let uniqueAccountId = account.uniqueAccountId
                print("Account deployed at address: \(address)")
                let deployedAccount = DeployedAccount(
                    info: accountInfo,
                    address: address,
                    uniqueAccountId: uniqueAccountId
                )
                try accountStore.accountDeployed(deployedAccount)

                withAnimation {
                    showingSuccess = true
                }

                try? await Task.sleep(for: .seconds(1.5))

                dismiss()

                deployedAddress = address

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
    PasskeyCreationView(
        challenge: Data("example-challenge".utf8),
        accountInfo: .init(
            name: "Jane Doe",
            userID: "jdoe",
            domain: "soo-sdk-example-pages.pages.dev"
        ),
        onDeployed: { _ in }
    )
}
