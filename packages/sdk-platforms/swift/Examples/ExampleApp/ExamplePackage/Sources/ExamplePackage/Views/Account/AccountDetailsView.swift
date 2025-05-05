import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

struct AccountDetailsView: View {
    @Environment(\.dismiss) private var dismiss

    @Environment(\.authorizationController) private var authorizationController

    @State private var account: AccountDetails
    @State private var isLoadingBalance = true
    @State private var isFunding = false
    @State private var showingCopiedFeedback = false
    @State private var showingSendTransaction = false
    @State private var showingLogoutConfirmation = false

    var onLogout: (() -> Void)?

    init(
        account: AccountDetails,
        onLogout: (() -> Void)? = nil
    ) {
        self.account = account
        self.onLogout = onLogout
    }

    var body: some View {
        ScrollView {
            VStack(spacing: 24) {
                VStack(spacing: 8) {
                    Text("Address")
                        .font(.headline)

                    Text(account.address)
                        .lineLimit(1)
                        .truncationMode(.middle)
                        .padding()
                        .background {
                            RoundedRectangle(cornerRadius: 12)
                                .fill(.secondary.opacity(0.1))
                        }
                        .onTapGesture {
                            UIPasteboard.general.string = account.address
                            withAnimation {
                                showingCopiedFeedback = true
                            }

                            Task {
                                try? await Task.sleep(for: .seconds(2))
                                withAnimation {
                                    showingCopiedFeedback = false
                                }
                            }
                        }
                        .overlay {
                            if showingCopiedFeedback {
                                Text("Copied!")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 4)
                                    .background(.thinMaterial)
                                    .cornerRadius(4)
                                    .transition(.move(edge: .top).combined(with: .opacity))
                            }
                        }

                    ActionButton(
                        title: "View on Explorer",
                        icon: "safari.fill",
                        style: .plain,
                        action: {
                            UIApplication.shared.open(account.explorerURL)
                        }
                    )
                }

                VStack(spacing: 8) {
                    Text("Account ID")
                        .font(.headline)

                    Text(account.uniqueAccountId)
                        .lineLimit(1)
                        .truncationMode(.middle)
                        .padding()
                        .background {
                            RoundedRectangle(cornerRadius: 12)
                                .fill(.secondary.opacity(0.1))
                        }
                        .onTapGesture {
                            UIPasteboard.general.string = account.uniqueAccountId
                            withAnimation {
                                showingCopiedFeedback = true
                            }

                            Task {
                                try? await Task.sleep(for: .seconds(2))
                                withAnimation {
                                    showingCopiedFeedback = false
                                }
                            }
                        }
                        .overlay {
                            if showingCopiedFeedback {
                                Text("Copied!")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 4)
                                    .background(.thinMaterial)
                                    .cornerRadius(4)
                                    .transition(.move(edge: .top).combined(with: .opacity))
                            }
                        }
                }

                VStack(spacing: 12) {
                    Text("Balance")
                        .font(.headline)

                    HStack {
                        Text(account.balance ?? "Loading...")
                            .font(.system(.title2, design: .monospaced))
                        if isLoadingBalance {
                            ProgressView()
                                .controlSize(.small)
                        }
                    }

                    Spacer()

                    ActionButton(
                        title: "Add Funds",
                        progressTitle: "Adding Funds...",
                        icon: "plus.circle.fill",
                        isLoading: isFunding,
                        style: .prominent,
                        action: {
                            Task {
                                await fundAccount()
                            }
                        }
                    )
                }

                ActionButton(
                    title: "Send Transaction",
                    icon: "paperplane.fill",
                    style: .prominent,
                    action: { showingSendTransaction = true }
                )
            }
            .padding()
        }
        .refreshable {
            await loadBalance()
        }
        .task {
            await loadBalance()
        }
        .navigationTitle("Account Details")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button {
                    showingLogoutConfirmation = true
                } label: {
                    Text("Logout")
                }
            }
        }
        .confirmationDialog(
            "Are you sure you want to log out?",
            isPresented: $showingLogoutConfirmation,
            titleVisibility: .visible
        ) {
            Button("Logout", role: .destructive) {
                onLogout?()
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("You can sign back in using your passkey.")
        }
        .sheet(isPresented: $showingSendTransaction) {
            SendTransactionView(
                fromAccount: account.account,
                onTransactionSent: {
                    Task {
                        await loadBalance()
                    }
                }
            )
        }
        .id("AccountDetailsView")
        .onAppear { print("AccountDetailsView appeared") }
    }

    private func loadBalance() async {
        isLoadingBalance = true
        defer { isLoadingBalance = false }

        do {
            let authenticator = PasskeyAuthenticatorHelper(
                controllerProvider: { self.authorizationController },
                relyingPartyIdentifier: "soo-sdk-example-pages.pages.dev"
            )
            
            let accountClient = AccountClient(
                account: .init(
                    address: account.address,
                    uniqueAccountId: account.uniqueAccountId
                ),
                authenticatorAsync: PasskeyAuthAsync(
                    authenticator: authenticator
                )
            )
            
            let balance = try await accountClient.getAccountBalance()
            self.account.balance = balance
        } catch {
            self.account.balance = "Error loading balance"
        }
    }

    private func fundAccount() async {
        guard !isFunding else { return }

        isFunding = true
        defer { isFunding = false }

        do {
            let authenticator = PasskeyAuthenticatorHelper(
                controllerProvider: { self.authorizationController },
                relyingPartyIdentifier: "soo-sdk-example-pages.pages.dev"
            )
            let accountClient = AccountClient(
                account: .init(
                    address: account.address,
                    uniqueAccountId: account.uniqueAccountId
                ),
                authenticatorAsync: PasskeyAuthAsync(
                    authenticator: authenticator
                )
            )
            
            try await accountClient.fundAccount()
            await loadBalance()
        } catch {
            print("Error funding account: \(error)")
        }
    }
}

#Preview {
    NavigationStack {
        AccountDetailsView(
            account: AccountDetails(
                account: .init(
                    info: .init(
                        name: "Jane Doe",
                        userID: "jdoe@example.com",
                        domain: "soo-sdk-example-pages.pages.dev"
                    ),
                    address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
                    uniqueAccountId: "jdoe@example.com"
                )
            )
        )
    }
}

#Preview("Dark Mode") {
    NavigationStack {
        AccountDetailsView(
            account: AccountDetails(
                account: .init(
                    info: .init(
                        name: "Jane Doe",
                        userID: "jdoe@example.com",
                        domain: "soo-sdk-example-pages.pages.dev"
                    ),
                    address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
                    uniqueAccountId: "jdoe@example.com"
                )
            )
        )
    }
    .preferredColorScheme(.dark)
}

#Preview("Different Widths") {
    NavigationStack {
        VStack(spacing: 20) {
            AccountDetailsView(
                account: AccountDetails(
                    account: .init(
                        info: .init(
                            name: "Jane Doe",
                            userID: "jdoe@example.com",
                            domain: "soo-sdk-example-pages.pages.dev"
                        ),
                        address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
                        uniqueAccountId: "jdoe@example.com"
                    )
                )
            )
            .frame(width: 300)

            AccountDetailsView(
                account: AccountDetails(
                    account: .init(
                        info: .init(
                            name: "Jane Doe",
                            userID: "jdoe@example.com",
                            domain: "soo-sdk-example-pages.pages.dev"
                        ),
                        address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
                        uniqueAccountId: "jdoe@example.com"
                    )
                )
            )
            .frame(width: 200)
        }
    }
}
