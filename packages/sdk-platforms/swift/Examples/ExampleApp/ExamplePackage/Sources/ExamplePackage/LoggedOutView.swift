import AuthenticationServices
import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

struct LoggedOutView: View {
    @Environment(\.authorizationController) private var authorizationController

    let accountInfo: AccountInfo

    @State private var showingCreateAccount = false
    @State private var showingLoginView = false

    var onAccountCreated: ((AccountDetails) -> Void)?
    var onSignedIn: ((AccountDetails) -> Void)?

    init(
        accountInfo: AccountInfo,
        onAccountCreated: ((AccountDetails) -> Void)? = nil,
        onSignedIn: ((AccountDetails) -> Void)? = nil
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
                    title: "Create Account",
                    icon: "plus.circle.fill",
                    style: .prominent
                ) {
                    showingCreateAccount = true
                }
                .sheet(isPresented: $showingCreateAccount) {
                    AccountCreationView(
                        accountInfo: accountInfo,
                        onDeployed: { deployedAccount in
                            if let onAccountCreated = onAccountCreated {
                                onAccountCreated(
                                    AccountDetails(
                                        account: deployedAccount,
                                        balance: nil
                                    )
                                )
                            }
                        }
                    )
                }

                ActionButton(
                    title: "Sign In",
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
            domain: "soo-sdk-example-pages.pages.dev"
        )
    )
}
