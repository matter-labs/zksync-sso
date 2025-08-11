import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

struct SessionDetailView: View {
    let session: Session
    let account: DeployedAccount
    let signers: AccountSigners

    @State private var showingRevokeConfirm = false
    @State private var isRevoking = false
    @State private var error: UIError?

    init(
        session: Session, account: DeployedAccount, signers: AccountSigners,
        error: UIError? = nil
    ) {
        self.session = session
        self.account = account
        self.signers = signers
        self._error = State(initialValue: error)
    }

    var body: some View {
        Form {
            Section {
                VStack(alignment: .leading, spacing: 16) {
                    Text("Session Details")
                        .font(.headline)

                    SessionSpecSummaryView(sessionSpec: session.sessionSpec)

                    SessionSpecDetailsJSONView(sessionSpec: session.sessionSpec)
                }
                .padding()
                .background(.background)
            }
            .listRowInsets(EdgeInsets())
            .listRowBackground(Color.clear)

            if let error = error {
                Section {
                    Text(error.message)
                        .foregroundStyle(.red)
                        .font(.footnote)
                }
                .listRowInsets(EdgeInsets())
                .listRowBackground(Color.clear)
            }

            Section {
                Button(action: { showingRevokeConfirm = true }) {
                    HStack(spacing: 8) {
                        Spacer()
                        if isRevoking {
                            ProgressView()
                                .padding(.trailing, 4)
                        }

                        Image(systemName: "trash.fill")

                        Text(isRevoking ? "Revoking Session..." : "Revoke Session")
                            .font(.headline)
                        Spacer()
                    }
                    .frame(maxWidth: .infinity)
                    .frame(height: 44)
                }
                .disabled(isRevoking)
                .buttonStyle(.borderedProminent)
                .tint(.red)
            }
            .listRowInsets(EdgeInsets())
            .listRowBackground(Color.clear)
        }
        .navigationTitle(shortHash(session.sessionHash))
        .confirmationDialog(
            "Revoke this session?",
            isPresented: $showingRevokeConfirm,
            titleVisibility: .visible
        ) {
            Button("Revoke", role: .destructive) {
                Task { await revoke() }
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("This action cannot be undone.")
        }
    }

    private func revoke() async {
        guard !isRevoking else { return }
        isRevoking = true
        error = nil
        defer { isRevoking = false }
        do {
            let args = RevokeSessionArgs(
                account: account.address,
                sessionId: session.id,
                ownerPrivateKey: signers.accountOwner.privateKeyHex
            )
            _ = try await revokeSession(args: args, config: .default)
        } catch {
            self.error = UIError(from: error)
        }
    }

    private func shortHash(_ hash: String) -> String {
        guard hash.count > 10 else { return hash }
        let start = hash.prefix(6)
        let end = hash.suffix(4)
        return String(start + "…" + end)
    }

    private func format(date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

#Preview {
    NavigationStack {
        SessionDetailView(
            session: .init(
                createdAt: Date(),
                sessionSpec: SessionSpec.default
            ),
            account: .init(
                info: .init(
                    name: "Jane Doe",
                    userID: "jdoe@example.com",
                    domain: "auth-test.zksync.dev"
                ),
                address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
                uniqueAccountId: "jdoe@example.com"
            ),
            signers: .default
        )
    }
}

#Preview("Error State") {
    NavigationStack {
        SessionDetailView(
            session: .init(
                createdAt: Date(),
                sessionSpec: SessionSpec.default
            ),
            account: .init(
                info: .init(
                    name: "Jane Doe",
                    userID: "jdoe@example.com",
                    domain: "auth-test.zksync.dev"
                ),
                address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
                uniqueAccountId: "jdoe@example.com"
            ),
            signers: .default,
            error: UIError(
                message:
                    "Failed to revoke session: ZKsyncSSOFFI.RevokeSessionError.RevokeSession(\"server returned an error response: error code 3: execution reverted: Error function_selector = 0x837529ed, data = 0x837529ed, data: \"0x837529ed\"\")"
            )
        )
    }
}
