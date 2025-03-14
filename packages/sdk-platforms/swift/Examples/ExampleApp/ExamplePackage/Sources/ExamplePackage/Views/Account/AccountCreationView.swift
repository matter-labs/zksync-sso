import SwiftUI

public struct AccountCreationView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var accountInfo: AccountInfo

    init(accountInfo: AccountInfo) {
        self.accountInfo = accountInfo
    }

    public var body: some View {
        NavigationStack {
            VStack(spacing: 32) {
                Spacer()

                VStack(spacing: 16) {
                    HStack {
                        Text("Username:")
                            .foregroundStyle(.secondary)
                        Spacer()
                        Text(accountInfo.name)
                            .fontWeight(.medium)
                    }

                    HStack {
                        Text("User ID:")
                            .foregroundStyle(.secondary)
                        Spacer()
                        Text(accountInfo.userID)
                            .font(.system(.body, design: .monospaced))
                            .fontWeight(.medium)
                            .lineLimit(1)
                            .minimumScaleFactor(0.5)
                    }
                }
                .padding()
                .frame(maxWidth: .infinity)
                .background {
                    RoundedRectangle(cornerRadius: 12)
                        .fill(.secondary.opacity(0.1))
                }
                .padding(.horizontal, 16)

                PasskeyCreationView(
                    challenge: Data((0..<32).map { _ in UInt8.random(in: 0...255) }),
                    accountInfo: accountInfo,
                    onDeployed: { deployedAccount in
                        dismiss()
                    }
                )

                Spacer()
            }
            .navigationTitle("Create Account")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }
}

#Preview {
    AccountCreationView(
        accountInfo: AccountInfo(
            name: "Jane Doe",
            userID: "jdoe@example.com",
            domain: "example.com"
        )
    )
}
