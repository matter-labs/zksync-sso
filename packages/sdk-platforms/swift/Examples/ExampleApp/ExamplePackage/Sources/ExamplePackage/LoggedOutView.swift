import AuthenticationServices
import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

struct LoggedOutView: View {
    @Environment(\.authorizationController) private var authorizationController

    let accountInfo: AccountInfo

    @State private var showingCreateAccount = false
    @State private var showingLoginView = false
    @State private var showingSessionDemoView = false

    var onAccountCreated: ((AccountSession) -> Void)?
    var onSignedIn: ((AccountSession) -> Void)?

    init(
        accountInfo: AccountInfo,
        onAccountCreated: ((AccountSession) -> Void)? = nil,
        onSignedIn: ((AccountSession) -> Void)? = nil
    ) {
        self.accountInfo = accountInfo
        self.onAccountCreated = onAccountCreated
        self.onSignedIn = onSignedIn
    }

    var body: some View {
        VStack(spacing: 32) {
            VStack(spacing: 16) {
                Image(systemName: "person.badge.key.fill")
                    .font(.system(size: 60))
                    .foregroundStyle(.tint)

                VStack(spacing: 8) {
                    Text("ZKsync SSO Example")
                        .font(.title2)
                        .fontWeight(.bold)

                    Text("Create an account or sign in with passkeys")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }
            }

            VStack(spacing: 16) {
                ActionButton(
                    title: "Create Account with Passkey",
                    icon: "plus.circle.fill",
                    style: .prominent
                ) {
                    showingCreateAccount = true
                }
                .sheet(isPresented: $showingCreateAccount) {
                    AccountCreationView(
                        accountInfo: accountInfo,
                        onDeployed: { deployedAccount, signers in
                            if let onAccountCreated = onAccountCreated {
                                onAccountCreated(
                                    AccountSession(
                                        accountDetails: AccountDetails(
                                            account: deployedAccount,
                                            balance: nil
                                        ),
                                        signers: signers
                                    )
                                )
                            }
                        }
                    )
                }

                ActionButton(
                    title: "Sign In with Passkey",
                    icon: "person.fill",
                    style: .plain
                ) {
                    showingLoginView = true
                }
                .sheet(isPresented: $showingLoginView) {
                    LoginView(
                        accountInfo: accountInfo,
                        onSignedIn: onSignedIn
                    )
                }

                if ExampleConfiguration.showSessionDemoView {
                    ActionButton(
                        title: "Session Demo",
                        icon: "wrench.and.screwdriver.fill",
                        style: .plain
                    ) {
                        showingSessionDemoView = true
                    }
                    .sheet(isPresented: $showingSessionDemoView) {
                        SessionDemoView()
                    }
                }
            }
        }
        .padding()
    }
}

#Preview {
    LoggedOutView(
        accountInfo: AccountInfo(
            name: "Jane Doe",
            userID: "jdoe@example.com",
            domain: "auth-test.zksync.dev"
        )
    )
}
