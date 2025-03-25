import ExamplePackageUIComponents
import SwiftUI

public struct AccountCreationView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var accountInfo: AccountInfo
    @FocusState private var focusedField: Field?
    @State private var showingSuccess: Bool
    let onDeployed: ((DeployedAccount) -> Void)?
    let previewWithRedBackground: Bool

    enum Field {
        case username
        case userID
    }

    init(
        accountInfo: AccountInfo,
        onDeployed: ((DeployedAccount) -> Void)? = nil,
        showToast: Bool = false,
        previewWithRedBackground: Bool = false
    ) {
        self.accountInfo = accountInfo
        self.onDeployed = onDeployed
        self._showingSuccess = State(initialValue: showToast)
        self.previewWithRedBackground = previewWithRedBackground
    }

    public var body: some View {
        NavigationStack {
            ZStack {
                // Main content
                VStack(spacing: 0) {
                    Spacer()

                    VStack(spacing: 24) {
                        Image(systemName: "person.badge.key.fill")
                            .font(.system(size: 60))
                            .foregroundStyle(.blue)
                            .background(previewWithRedBackground ? Color.red.opacity(0.8) : nil)

                        VStack(alignment: .leading, spacing: 8) {
                            Text("Username")
                                .foregroundStyle(.secondary)
                                .font(.subheadline)

                            ZStack(alignment: .trailing) {
                                TextField("Enter username", text: $accountInfo.name)
                                    .focused($focusedField, equals: .username)
                                    .padding()
                                    .background(Color(.systemGray6))
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
                                value: focusedField == .username && !accountInfo.name.isEmpty)
                        }

                        VStack(alignment: .leading, spacing: 8) {
                            Text("User ID")
                                .foregroundStyle(.secondary)
                                .font(.subheadline)

                            ZStack(alignment: .trailing) {
                                TextField("Enter user ID", text: $accountInfo.userID)
                                    .focused($focusedField, equals: .userID)
                                    .padding()
                                    .background(Color(.systemGray6))
                                    .cornerRadius(10)
                                    .font(.system(.body, design: .monospaced))
                                    .autocorrectionDisabled()
                                    .textInputAutocapitalization(.never)

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
                                value: focusedField == .userID && !accountInfo.userID.isEmpty)
                        }

                        VStack {
                            if !previewWithRedBackground {
                                AccountCreationContentView(
                                    challenge: Data(
                                        (0..<32).map { _ in UInt8.random(in: 0...255) }),
                                    accountInfo: accountInfo,
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
                                    onDeployed: { deployedAccount in
                                        onDeployed?(deployedAccount)
                                    }
                                )
                            } else {
                                Button {
                                } label: {
                                    HStack {
                                        Image(systemName: "key.fill")
                                            .font(.system(size: 16))
                                        Text("Create Passkey")
                                            .font(.headline)
                                    }
                                    .frame(maxWidth: .infinity)
                                    .frame(height: 50)
                                    .background(Color.blue)
                                    .foregroundColor(.white)
                                    .cornerRadius(10)
                                }
                            }
                        }
                        .padding(.top, 16)
                    }
                    .padding(.horizontal, 24)

                    Spacer()
                }
                .background(previewWithRedBackground ? Color.red.opacity(0.2) : nil)

                // Toast overlay
                if showingSuccess {
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
            }
            .navigationTitle("Create Account")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Cancel") {
                        if !previewWithRedBackground {
                            dismiss()
                        }
                    }
                }

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
