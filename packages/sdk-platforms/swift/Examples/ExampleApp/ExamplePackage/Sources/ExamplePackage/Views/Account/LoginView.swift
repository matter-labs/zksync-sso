import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

public struct LoginView: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(\.authorizationController) private var authorizationController
    @State private var accountInfo: AccountInfo
    @FocusState private var isFocused: Bool
    @State private var isSigningIn = false
    @State private var error: UIError?

    let onSignedIn: ((AccountSession) -> Void)?

    init(
        accountInfo: AccountInfo,
        onSignedIn: ((AccountSession) -> Void)? = nil
    ) {
        self.accountInfo = accountInfo
        self.onSignedIn = onSignedIn
    }

    public var body: some View {
        NavigationStack {
            VStack(spacing: 32) {
                Spacer()

                Image(systemName: "person.badge.key.fill")
                    .font(.system(size: 70))
                    .foregroundStyle(.tint)
                    .padding(.bottom, 20)

                VStack(spacing: 20) {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("User ID")
                            .foregroundStyle(.secondary)
                            .font(.subheadline)

                        ZStack(alignment: .trailing) {
                            TextField("Enter your user ID", text: $accountInfo.userID)
                                .focused($isFocused)
                                .padding()
                                .background(Color(uiColor: .systemGray6))
                                .cornerRadius(10)
                                .font(.system(.body, design: .monospaced))
                                .autocorrectionDisabled()
                                .autocapitalization(.none)

                            if isFocused && !accountInfo.userID.isEmpty {
                                Button {
                                    withAnimation {
                                        accountInfo.userID = ""
                                    }
                                } label: {
                                    Image(systemName: "xmark.circle.fill")
                                        .foregroundStyle(.gray)
                                }
                                .padding(.trailing, 16)
                                .transition(.opacity)
                            }
                        }
                        .animation(
                            .easeInOut(duration: 0.2),
                            value: isFocused && !accountInfo.userID.isEmpty
                        )
                    }

                    if let error = error {
                        Text(error.message)
                            .foregroundStyle(.red)
                            .font(.footnote)
                            .padding(.top, 4)
                    }

                    ActionButton(
                        title: "Sign In",
                        progressTitle: "Signing In...",
                        icon: "person.fill",
                        isLoading: isSigningIn,
                        style: .prominent,
                        action: signIn
                    )
                    .padding(.top, 16)
                }
                .padding(.horizontal, 16)

                Spacer()
            }
            .navigationTitle("Sign In")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Cancel") {
                        dismiss()
                    }
                }

                ToolbarItem(placement: .keyboard) {
                    Button("Done") {
                        isFocused = false
                    }
                }
            }
        }
    }

    private func signIn() {
        guard !accountInfo.userID.isEmpty else {
            error = UIError(message: "Please enter your user ID")
            return
        }

        isSigningIn = true
        error = nil

        Task {
            defer { isSigningIn = false }

            do {
                let uniqueAccountId = accountInfo.userID
                let relyingPartyIdentifier = accountInfo.domain

                let account = try await getAccountByUserId(
                    uniqueAccountId: uniqueAccountId,
                    relyingPartyIdentifier: relyingPartyIdentifier,
                )

                let accountDetails = AccountDetails(
                    account: .init(
                        info: accountInfo,
                        address: account.address,
                        uniqueAccountId: account.uniqueAccountId
                    )
                )

                onSignedIn?(
                    AccountSession(
                        accountDetails: accountDetails,
                        signers: .default
                    ))
                dismiss()
            } catch {
                self.error = UIError(from: error)
                print("Sign in error: \(error)")
            }
        }
    }
}

#Preview {
    LoginView(
        accountInfo: AccountInfo(
            name: "Jane Doe",
            userID: "jdoe@example.com",
            domain: "example.com"
        )
    )
}
