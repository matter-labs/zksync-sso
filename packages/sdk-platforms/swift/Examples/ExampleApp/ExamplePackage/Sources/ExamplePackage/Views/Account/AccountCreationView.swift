import ExamplePackageUIComponents
import SwiftUI

typealias OnDeployed = (DeployedAccount, AccountSigners) -> Void

public struct AccountCreationView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var accountInfo: AccountInfo
    @FocusState private var focusedField: Field?
    @State private var showingSuccess: Bool
    let onDeployed: OnDeployed?
    let previewWithRedBackground: Bool

    enum Field {
        case username
        case userID
    }

    init(
        accountInfo: AccountInfo,
        onDeployed: OnDeployed? = nil,
        showToast: Bool = false,
        previewWithRedBackground: Bool = false
    ) {
        self.accountInfo = accountInfo
        self.onDeployed = onDeployed
        self._showingSuccess = State(initialValue: showToast)
        self.previewWithRedBackground = previewWithRedBackground
    }

    private var usernameField: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Username")
                .foregroundStyle(.secondary)
                .font(.subheadline)

            ZStack(alignment: .trailing) {
                TextField("Enter username", text: $accountInfo.name)
                    .focused($focusedField, equals: .username)
                    .padding()
                    .background(Color.secondary.opacity(0.1))
                    .cornerRadius(10)
                    .autocorrectionDisabled()

                if focusedField == .username && !accountInfo.name.isEmpty {
                    Button {
                        withAnimation {
                            accountInfo.name = ""
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
                value: focusedField == .username && !accountInfo.name.isEmpty
            )
        }
    }

    private var userIDField: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("User ID")
                .foregroundStyle(.secondary)
                .font(.subheadline)

            ZStack(alignment: .trailing) {
                TextField("Enter user ID", text: $accountInfo.userID)
                    .focused($focusedField, equals: .userID)
                    .padding()
                    .background(Color.secondary.opacity(0.1))
                    .cornerRadius(10)
                    .font(.system(.body, design: .monospaced))
                    .autocorrectionDisabled()

                if focusedField == .userID && !accountInfo.userID.isEmpty {
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
                value: focusedField == .userID && !accountInfo.userID.isEmpty
            )
        }
    }

    private var mainFormContent: some View {
        VStack(spacing: 24) {
            Image(systemName: "person.badge.key.fill")
                .font(.system(size: 60))
                .foregroundStyle(.blue)
                .background(previewWithRedBackground ? Color.red.opacity(0.8) : nil)

            usernameField
            userIDField

            contentView
        }
    }

    private var contentView: some View {
        VStack {
            if !previewWithRedBackground {
                AccountCreationContentView(
                    challenge: Data(
                        (0..<32).map { _ in UInt8.random(in: 0...255) }),
                    accountInfo: accountInfo,
                    signers: .default,
                    onSuccess: {
                        withAnimation {
                            showingSuccess = true
                        }

                        // Dismiss after delay
                        Task {
                            try? await Task.sleep(for: .seconds(1.5))
                            dismiss()
                        }
                    },
                    onDeployed: { deployedAccount, signer in
                        onDeployed?(deployedAccount, signer)
                    }
                )
                .environmentObject(SessionsStore.shared)
            } else {
                Button {
                } label: {
                    HStack {
                        Image(systemName: "key.fill")
                            .foregroundStyle(.white)

                        Text("Create Account")
                            .foregroundStyle(.white)
                            .font(.headline)
                    }
                    .frame(maxWidth: .infinity)
                    .frame(height: 50)
                }
                .background(Color.red)
                .cornerRadius(10)
            }
        }
    }

    private var successOverlay: some View {
        Color.black.opacity(0.001)  // Invisible background to capture whole screen
            .edgesIgnoringSafeArea(.all)
            .overlay {
                ToastView(
                    icon: "checkmark.circle.fill",
                    iconColor: .green,
                    message: "Account Deployed!"
                )
            }
    }

    public var body: some View {
        NavigationStack {
            ZStack {
                // Main content
                VStack(spacing: 0) {
                    Spacer()
                    mainFormContent
                    Spacer()
                }
                .padding(.horizontal, 24)
                .opacity(showingSuccess ? 0 : 1)

                // Toast overlay
                if showingSuccess {
                    successOverlay
                }
            }
            .navigationTitle("Create Account")
            #if os(iOS)
                .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                #if os(iOS)
                    ToolbarItem(placement: .navigationBarTrailing) {
                        Button("Cancel") {
                            if !previewWithRedBackground {
                                dismiss()
                            }
                        }
                    }
                #else
                    ToolbarItem(placement: .primaryAction) {
                        Button("Cancel") {
                            if !previewWithRedBackground {
                                dismiss()
                            }
                        }
                    }
                #endif

                ToolbarItem(placement: .keyboard) {
                    Button("Done") {
                        focusedField = nil
                    }
                }
            }
        }
    }
}

#Preview("Default") {
    AccountCreationView(
        accountInfo: AccountInfo(
            name: "Jane Doe",
            userID: "jdoe@example.com",
            domain: "example.com"
        )
    )
}

#Preview("With Success Toast") {
    AccountCreationView(
        accountInfo: AccountInfo(
            name: "Jane Doe",
            userID: "jdoe@example.com",
            domain: "example.com"
        ),
        showToast: true,
        previewWithRedBackground: true
    )
}
