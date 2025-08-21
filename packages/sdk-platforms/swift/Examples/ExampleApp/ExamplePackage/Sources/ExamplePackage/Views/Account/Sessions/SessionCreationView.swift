import SwiftUI
import ZKsyncSSO
import ZKsyncSSOIntegration

struct SessionCreationView: View {
    let deployedAccount: DeployedAccountDetails
    let signers: AccountSigners
    let onSessionCreated: (Session) -> Void

    @Environment(\.dismiss) private var dismiss
    @State private var sessionSpec: SessionSpec
    @State private var isCreating = false
    @State private var error: UIError?

    init(
        deployedAccount: DeployedAccountDetails,
        signers: AccountSigners,
        onSessionCreated: @escaping (Session) -> Void,
        error: UIError? = nil
    ) {
        self.deployedAccount = deployedAccount
        self.signers = signers
        self.onSessionCreated = onSessionCreated
        self._error = State(initialValue: error)

        let sessionConfig = SessionSpec(
            signer: "0x90F79bf6EB2c4f870365E785982E1f101E93b906",  // secondSessionOwnerAddress from debug
            expiresAt: "1767225600",  // January 1st, 2026 00:00:00 UTC
            feeLimit: UsageLimit(
                limitType: .lifetime,
                limit: "50000000000000000",  // 0.05 ETH
                period: "0"
            ),
            callPolicies: [],
            transferPolicies: [
                TransferSpec(
                    target: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",  // vitalikAddress
                    maxValuePerUse: "5000000000000000",  // 0.005 ETH
                    valueLimit: UsageLimit(
                        limitType: .unlimited,
                        limit: "0",
                        period: "0"
                    )
                )
            ]
        )
        self._sessionSpec = State(initialValue: sessionConfig)
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
            let sessionOwner = signers.sessionOwner
            let sessionKey = sessionOwner.privateKeyHex
            
            // Assert we're using the correct account address (should be the deterministic one)
            assert(
                deployedAccount.address == IntegrationConstants.deployedAccountAddress,
                "Account address mismatch: expected \(IntegrationConstants.deployedAccountAddress), got \(deployedAccount.address)"
            )
            
            print("🚀 Creating session using ZKsyncSSOIntegration.createSession...")
            
            let sessionHashStr = try await ZKsyncSSOIntegration.createSession(
                deployedAccount: deployedAccount
            )
            
            print("✅ Session creation completed!")
            print("  Session ID: \(sessionHashStr)")
            
            let newSession = Session(
                createdAt: Date(),
                sessionSpec: sessionSpec,
                sessionKey: sessionKey
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
        deployedAccount: DeployedAccountDetails.default,
        signers: .default,
        onSessionCreated: { _ in
 }
    )
}

#Preview("Error State") {
    SessionCreationView(
        deployedAccount: DeployedAccountDetails.default,
        signers: .default,
        onSessionCreated: { _ in },
        error: UIError(
            message:
                "Failed to create session: ZKsyncSSOFFI.CreateSessionError.CreateSession(\"server returned an error response: error code 3: execution reverted: Error function_selector = 0x12345678, data = 0x12345678, data: \"0x12345678\"\")"
        )
    )
}
