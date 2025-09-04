import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO
import ZKsyncSSOIntegration

#if canImport(UIKit)
    import UIKit
#endif

struct AccountDetailsView: View {
    @Environment(\.dismiss) private var dismiss

    @Environment(\.authorizationController) private var authorizationController

    @State private var account: AccountDetails
    @State private var isLoadingBalance = true
    @State private var isFunding = false
    @State private var showingCopiedFeedback = false
    @State private var showingSendTransaction = false
    @State private var showingSessions = false
    @State private var showingLogoutConfirmation = false

    let signers: AccountSigners
    var onLogout: (() -> Void)?

    init(
        account: AccountDetails,
        signers: AccountSigners = .default,
        onLogout: (() -> Void)? = nil
    ) {
        self.account = account
        self.signers = signers
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
                            #if canImport(UIKit)
                                UIPasteboard.general.string = account.address
                            #endif
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
                            #if canImport(UIKit)
                                UIApplication.shared.open(account.explorerURL)
                            #endif
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
                            #if canImport(UIKit)
                                UIPasteboard.general.string = account.uniqueAccountId
                            #endif
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
                
                if ExampleConfiguration.showSessionAccountFlowView {
                    VStack(spacing: 12) {
                        Text("Sessions")
                            .font(.headline)
                        
                        ActionButton(
                            title: "Sessions",
                            icon: "list.bullet.rectangle.portrait",
                            style: .plain,
                            action: { showingSessions = true }
                        )
                    }
                }
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
        #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
        #endif
        #if os(iOS)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button {
                        showingLogoutConfirmation = true
                    } label: {
                        Text("Logout")
                    }
                }
            }
        #endif
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
                configuration: TransactionConfigurationFactory.regularTransaction(
                    fromAccount: account.account,
                    authorizationController: authorizationController
                ),
                fromAddress: account.account.address,
                onTransactionSent: {
                    Task {
                        await loadBalance()
                    }
                }
            )
        }
        .sheet(isPresented: $showingSessions) {
            SessionsView(
                account: DeployedAccountDetails(
                    address: account.address,
                    owner: signers.accountOwner,
                    uniqueAccountId: account.uniqueAccountId,
                    sessionConfigJson: nil,
                    config: .default
                ),
                signers: signers
            )
            .environmentObject(SessionsStore.shared)
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
                relyingPartyIdentifier: "auth-test.zksync.dev"
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
            account.balance = balance
        } catch {
            account.balance = "Error loading balance"
        }
    }

    private func fundAccount() async {
        guard !isFunding else { return }

        isFunding = true
        defer { isFunding = false }

        do {
            let authenticator = PasskeyAuthenticatorHelper(
                controllerProvider: { self.authorizationController },
                relyingPartyIdentifier: "auth-test.zksync.dev"
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

            try await accountClient.fundAccount(amount: "1.0")
            
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
                        domain: "auth-test.zksync.dev"
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
                        domain: "auth-test.zksync.dev"
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
                            domain: "auth-test.zksync.dev"
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
                            domain: "auth-test.zksync.dev"
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
