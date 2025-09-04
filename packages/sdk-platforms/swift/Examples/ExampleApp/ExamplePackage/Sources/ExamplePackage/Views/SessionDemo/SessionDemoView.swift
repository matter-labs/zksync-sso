import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO
import ZKsyncSSOIntegration

enum DebugMenuState {
    case initial
    case accountDeployed(deployedAccount: DeployedAccountDetails)
    case sessionCreatedAndRevoked
    case sessionTransactionSent
    case error(UIError)
    case progress(ActionInProgress)
}

struct ActionInProgress {
    let title: String
    let message: String
    let action: String
}

struct SuccessToastConfig {
    let icon: String
    let iconColor: Color = .green
    let message: String
}

struct FailureToastConfig: Error {
    let icon: String
    let iconColor: Color = .red
    let message: String
}

struct SessionDemoView: View {
    @Environment(\.dismiss) private var dismiss

    @State private var actionResultToast: Result<SuccessToastConfig, FailureToastConfig>?
    @State private var menuState: DebugMenuState

    init(initialState: DebugMenuState = .initial) {
        self._menuState = State(initialValue: initialState)
    }

    var body: some View {
        NavigationView {
            VStack(spacing: 24) {
                VStack(spacing: 16) {
                    Image(systemName: "wrench.and.screwdriver.fill")
                        .font(.system(size: 50))
                        .foregroundStyle(.orange)

                    Text("Debug Actions")
                        .font(.title2)
                        .fontWeight(.bold)

                    Text("Test ZKsyncSSO SDK functionality")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                switch menuState {
                case .initial:
                    initialView
                case .accountDeployed(let deployedAccount):
                    accountDeployedView(deployedAccount: deployedAccount)
                case .sessionCreatedAndRevoked:
                    sessionCreatedAndRevokedView
                case .sessionTransactionSent:
                    sessionTransactionSentView
                case .error(let error):
                    errorView(error)
                case .progress(let progress):
                    progressView(progress)
                }

                Spacer()
            }
            .padding()
            .navigationTitle("Debug")
            #if os(iOS)
                .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                #if os(iOS)
                    ToolbarItem(placement: .navigationBarTrailing) {
                        Button("Done") {
                            dismiss()
                        }
                    }
                #else
                    ToolbarItem(placement: .primaryAction) {
                        Button("Done") {
                            dismiss()
                        }
                    }
                #endif
            }
            .overlay {
                if let result = actionResultToast {
                    switch result {
                    case .success(let config):
                        ToastView(
                            icon: config.icon,
                            iconColor: config.iconColor,
                            message: config.message
                        )
                    case .failure(let config):
                        ToastView(
                            icon: config.icon,
                            iconColor: config.iconColor,
                            message: config.message
                        )
                    }
                }
            }
        }
    }

    private var initialView: some View {
        VStack(spacing: 16) {
            ActionButton(
                title: "Deploy Account",
                icon: "plus.circle.fill",
                style: .prominent
            ) {
                deployAccount()
            }
        }
    }

    private func accountDeployedView(
        deployedAccount: DeployedAccountDetails
    ) -> some View {
        VStack(spacing: 16) {
            VStack(spacing: 8) {
                Text("Account Deployed Successfully!")
                    .font(.headline)
                    .fontWeight(.bold)
                    .foregroundStyle(.green)

                Text("Address:")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                Button {
                    #if os(iOS)
                    UIPasteboard.general.string = deployedAccount.address
                    #elseif os(macOS)
                        NSPasteboard.general.clearContents()
                        NSPasteboard.general.setString(address, forType: .string)
                    #endif
                } label: {
                    HStack {
                        Text(deployedAccount.address)
                            .font(.system(.body, design: .monospaced))
                            .fontWeight(.medium)
                            .foregroundStyle(.primary)

                        Image(systemName: "doc.on.doc")
                            .font(.system(size: 14))
                            .foregroundStyle(.secondary)
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                    .background(Color.secondary.opacity(0.1))
                    .cornerRadius(8)
                }
                .buttonStyle(.plain)
            }

            ActionButton(
                title: "Create and Revoke Session",
                icon: "key.fill",
                style: .prominent
            ) {
                sessionCreateAndRevoke(deployedAccount: deployedAccount)
            }
        }
    }

    private var sessionCreatedAndRevokedView: some View {
        VStack(spacing: 16) {
            ActionButton(
                title: "Send Transaction",
                icon: "paperplane.fill",
                style: .prominent
            ) {
                sessionSendTransaction()
            }
        }
    }

    private var sessionTransactionSentView: some View {
        VStack(spacing: 16) {
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 60))
                .foregroundStyle(.green)

            Text("All Actions Successful!")
                .font(.title2)
                .fontWeight(.bold)

            Text(
                "Account deployed, session created/revoked, and session transaction sent successfully"
            )
            .font(.subheadline)
            .foregroundStyle(.secondary)
            .multilineTextAlignment(.center)
        }
    }

    private var successView: some View {
        VStack(spacing: 16) {
            Image(systemName: "checkmark.circle.fill")
                .font(.system(size: 60))
                .foregroundStyle(.green)

            Text("All Actions Successful!")
                .font(.title2)
                .fontWeight(.bold)

            Text("Account deployed and session created/revoked successfully")
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
    }

    private func errorView(_ error: UIError) -> some View {
        VStack(spacing: 16) {
            Image(systemName: "xmark.circle.fill")
                .font(.system(size: 60))
                .foregroundStyle(.red)

            Text("Error Occurred")
                .font(.title2)
                .fontWeight(.bold)

            Text(error.message)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            ActionButton(
                title: "Try Again",
                icon: "arrow.clockwise",
                style: .plain
            ) {
                menuState = .initial
            }
        }
    }

    private func progressView(_ progress: ActionInProgress) -> some View {
        VStack(spacing: 16) {
            ProgressView()
                .scaleEffect(1.5)
                .padding(.vertical, 8)

            Text(progress.title)
                .font(.headline)
                .fontWeight(.bold)

            Text(progress.message)
                .font(.subheadline)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
    }

    private func deployAccount() {
        withAnimation {
            menuState = .progress(
                ActionInProgress(
                    title: "Deploying Account",
                    message: "Creating and funding your account on zkSync...",
                    action: "Deploy Account"
                ))
        }

        Task {
            do {
                let deployedAccount = try await SessionDemoActions.deployAccount()

                await MainActor.run {
                    showToast(
                        Result.success(
                            SuccessToastConfig(
                                icon: "checkmark.circle.fill",
                                message: "Account Deployed Successfully!"
                            )))

                    withAnimation {
                        menuState = .accountDeployed(
                            deployedAccount: deployedAccount
                        )
                    }
                }
            } catch {
                await MainActor.run {
                    let uiError = UIError(from: error)
                    showToast(
                        Result.failure(
                            FailureToastConfig(
                                icon: "xmark.circle.fill",
                                message:
                                    "Account Deployment Failed: \(error.localizedDescription)"
                            )))

                    withAnimation {
                        menuState = .error(uiError)
                    }
                }
            }
        }
    }

    private func sessionCreateAndRevoke(
        deployedAccount: DeployedAccountDetails
    ) {
        withAnimation {
            menuState = .progress(
                ActionInProgress(
                    title: "Creating and Revoking Session",
                    message: "Setting up session management and testing revocation...",
                    action: "Create and Revoke Session"
                ))
        }

        Task {
            do {
                try await SessionDemoActions.createAndRevokeSession(
                    deployedAccount: deployedAccount
                )

                await MainActor.run {
                    showToast(
                        Result.success(
                            SuccessToastConfig(
                                icon: "checkmark.circle.fill",
                                message: "Session Created and Revoked Successfully!"
                            )))

                    withAnimation {
                        menuState = .sessionCreatedAndRevoked
                    }
                }
            } catch {
                await MainActor.run {
                    let uiError = UIError(from: error)
                    showToast(
                        Result.failure(
                            FailureToastConfig(
                                icon: "xmark.circle.fill",
                                message:
                                    "Session Create and Revoke Failed: \(error.localizedDescription)"
                            )))

                    withAnimation {
                        menuState = .error(uiError)
                    }
                }
            }
        }
    }

    private func sessionSendTransaction() {
        withAnimation {
            menuState = .progress(
                ActionInProgress(
                    title: "Sending Session Transaction",
                    message: "Executing session transaction on zkSync...",
                    action: "Send Session Transaction"
                ))
        }

        Task {
            do {
                try await SessionDemoActions.sessionSendTransaction()

                await MainActor.run {
                    showToast(
                        Result.success(
                            SuccessToastConfig(
                                icon: "checkmark.circle.fill",
                                message: "Session Transaction Sent Successfully!"
                            )))

                    withAnimation {
                        menuState = .sessionTransactionSent
                    }
                }
            } catch {
                await MainActor.run {
                    let uiError = UIError(from: error)
                    showToast(
                        Result.failure(
                            FailureToastConfig(
                                icon: "xmark.circle.fill",
                                message:
                                    "Session Transaction Failed: \(error.localizedDescription)"
                            )))

                    withAnimation {
                        menuState = .error(uiError)
                    }
                }
            }
        }
    }

    private func showToast(_ result: Result<SuccessToastConfig, FailureToastConfig>) {
        withAnimation {
            actionResultToast = result
        }

        // Hide toast after 1.5 seconds
        Task {
            try? await Task.sleep(for: .seconds(1.5))
            withAnimation {
                actionResultToast = nil
            }
        }
    }
}

#Preview("Initial State") {
    SessionDemoView(initialState: .initial)
}

#Preview("Account Deployed") {
    SessionDemoView(initialState: .accountDeployed(deployedAccount: .default))
}

#Preview("Session Created and Revoked") {
    SessionDemoView(initialState: .sessionCreatedAndRevoked)
}

#Preview("Transaction Sent") {
    SessionDemoView(initialState: .sessionTransactionSent)
}

#Preview("Error State") {
    SessionDemoView(initialState: .error(UIError(message: "Network connection failed")))
}

#Preview("Progress State - Deploying") {
    SessionDemoView(
        initialState: .progress(
            ActionInProgress(
                title: "Deploying Account",
                message: "Creating and funding your account on zkSync...",
                action: "Deploy Account"
            )))
}

#Preview("Progress State - Session") {
    SessionDemoView(
        initialState: .progress(
            ActionInProgress(
                title: "Creating and Revoking Session",
                message: "Setting up session management and testing revocation...",
                action: "Create and Revoke Session"
            )))
}

#Preview("Progress State - Session Transaction") {
    SessionDemoView(
        initialState: .progress(
            ActionInProgress(
                title: "Sending Session Transaction",
                message: "Executing session transaction on zkSync...",
                action: "Send Session Transaction"
            )))
}
