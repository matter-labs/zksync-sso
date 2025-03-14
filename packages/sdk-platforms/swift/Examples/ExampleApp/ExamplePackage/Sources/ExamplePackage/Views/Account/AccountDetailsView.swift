import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

struct AccountDetailsView: View {
    @Environment(\.dismiss) private var dismiss

    @State private var account: AccountDetails
    @State private var isLoadingBalance = true
    @State private var isFunding = false
    @State private var showingCopiedFeedback = false
    @State private var showingSendTransaction = false

    private let accountClient: AccountClient

    private let passkeyAuth: PasskeyAuthSync

    init(address: String, uniqueAccountId: String, relyingPartyIdentifier: String) {
        let passkeyAuth = PasskeyAuthSync(
            authenticator: PasskeyAuthenticatorHelper(
                manager: PasskeyManager(
                    relyingPartyIdentifier: relyingPartyIdentifier
                )
            )
        )
        self.account = AccountDetails(
            address: address,
            uniqueAccountId: uniqueAccountId
        )
        self.accountClient = AccountClient(
            account: .init(address: address, uniqueAccountId: uniqueAccountId),
            authenticator: passkeyAuth
        )
        self.passkeyAuth = passkeyAuth
    }

    init(account: AccountDetails, relyingPartyIdentifier: String) {
        let passkeyAuth = PasskeyAuthSync(
            authenticator: PasskeyAuthenticatorHelper(
                manager: PasskeyManager(
                    relyingPartyIdentifier: relyingPartyIdentifier
                )
            )
        )
        self.account = account
        self.accountClient = AccountClient(
            account: .init(
                address: account.address,
                uniqueAccountId: account.uniqueAccountId
            ),
            authenticator: passkeyAuth
        )
        self.passkeyAuth = passkeyAuth
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
                        action: { UIApplication.shared.open(account.explorerURL) }
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
        .sheet(isPresented: $showingSendTransaction) {
            SendTransactionView(
                fromAccount: account,
                passkeyAuth: passkeyAuth,
                onTransactionSent: {
                    Task {
                        await loadBalance()
                    }
                }
            )
        }
        .passkeyPresentation(passkeyAuth.manager)
        .id("AccountDetailsView")
        .onAppear { print("AccountDetailsView appeared") }
    }

    private func loadBalance() async {
        isLoadingBalance = true
        defer { isLoadingBalance = false }

        do {
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
            address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            uniqueAccountId: "uniqueAccountId",
            relyingPartyIdentifier: "example.com"
        )
    }
}

#Preview("Dark Mode") {
    NavigationStack {
        AccountDetailsView(
            address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            uniqueAccountId: "uniqueAccountId",
            relyingPartyIdentifier: "example.com"
        )
    }
    .preferredColorScheme(.dark)
}

#Preview("Different Widths") {
    NavigationStack {
        VStack(spacing: 20) {
            AccountDetailsView(
                address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
                uniqueAccountId: "uniqueAccountId",
                relyingPartyIdentifier: "example.com"
            )
            .frame(width: 300)

            AccountDetailsView(
                address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
                uniqueAccountId: "uniqueAccountId",
                relyingPartyIdentifier: "example.com"
            )
            .frame(width: 200)
        }
    }
}
