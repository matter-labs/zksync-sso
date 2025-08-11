import SwiftUI
import ZKsyncSSO

struct SessionCreationView: View {
    let accountAddress: String
    let signers: AccountSigners
    let onSessionCreated: (Session) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var sessionSpec = SessionSpec.default
    @State private var isCreating = false
    @State private var error: UIError?

    init(
        accountAddress: String,
        signers: AccountSigners,
        onSessionCreated: @escaping (Session) -> Void,
        error: UIError? = nil
    ) {
        self.accountAddress = accountAddress
        self.signers = signers
        self.onSessionCreated = onSessionCreated
        self._error = State(initialValue: error)
    }

    private var sessionConfigJson: String {
        try! sessionSpec.toJsonString(pretty: true)
    }

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    SessionSpecDetailsView(sessionSpec: sessionSpec)
                }
                .listRowInsets(EdgeInsets())

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
                    Button(action: { Task { await createSession() } }) {
                        HStack(spacing: 8) {
                            Spacer()
                            if isCreating {
                                ProgressView()
                                    .padding(.trailing, 4)
                            }

                            Image(systemName: "plus.circle.fill")

                            Text(isCreating ? "Creating Session..." : "Create Session")
                                .font(.headline)
                            Spacer()
                        }
                        .frame(maxWidth: .infinity)
                        .frame(height: 44)
                    }
                    .disabled(isCreating)
                    .buttonStyle(.borderedProminent)
                }
                .listRowInsets(EdgeInsets())
                .listRowBackground(Color.clear)
            }
            .navigationTitle("New Session")
            #if os(iOS)
                .navigationBarTitleDisplayMode(.inline)
            #endif
            #if os(iOS)
                .toolbar {
                    ToolbarItem(placement: .navigationBarLeading) {
                        Button("Cancel") {
                            dismiss()
                        }
                        .disabled(isCreating)
                    }
                }
            #endif
        }
    }

    private func createSession() async {
        guard !isCreating else { return }
        isCreating = true
        error = nil
        defer { isCreating = false }

        do {
            let args = CreateSessionArgs(
                account: accountAddress,
                sessionConfig: sessionSpec,
                ownerPrivateKey: signers.accountOwner.privateKeyHex,
                paymaster: nil
            )
            _ = try await ZKsyncSSO.createSession(
                args: args,
                config: Config.default
            )

            let newSession = Session(
                createdAt: Date(),
                sessionSpec: sessionSpec
            )
            onSessionCreated(newSession)

            dismiss()
        } catch {
            self.error = UIError(from: error)
            print("Failed to create session: \(error)")
        }
    }
}

#Preview {
    SessionCreationView(
        accountAddress: "0x1234567890abcdef",
        signers: .default,
        onSessionCreated: { _ in }
    )
}

#Preview("Error State") {
    SessionCreationView(
        accountAddress: "0x1234567890abcdef",
        signers: .default,
        onSessionCreated: { _ in },
        error: UIError(
            message:
                "Failed to create session: ZKsyncSSOFFI.CreateSessionError.CreateSession(\"server returned an error response: error code 3: execution reverted: Error function_selector = 0x12345678, data = 0x12345678, data: \"0x12345678\"\")"
        )
    )
}
