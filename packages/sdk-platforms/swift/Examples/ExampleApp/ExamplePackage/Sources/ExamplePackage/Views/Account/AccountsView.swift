import SwiftUI

public struct AccountsView: View {
    @State private var accountStore = AccountStore()
    @State private var showingDeploySheet = false
    
    let relyingPartyIdentifier: String

    public init(relyingPartyIdentifier: String) {
        self.relyingPartyIdentifier = relyingPartyIdentifier
    }

    public var body: some View {
        List {
            Section("Deployed Accounts") {
                ForEach(accountStore.deployedAccounts) { account in
                    AccountRowView(
                        account: account,
                        relyingPartyIdentifier: relyingPartyIdentifier
                    )
                }
            }
        }
        .overlay {
            if accountStore.deployedAccounts.isEmpty {
                VStack(spacing: 16) {
                    ContentUnavailableView(
                        "No Accounts",
                        systemImage: "person.crop.circle.badge.exclamationmark",
                        description: Text("Create your first account to get started")
                    )

                    Button {
                        showingDeploySheet = true
                    } label: {
                        Label("Create Account", systemImage: "plus.circle.fill")
                            .font(.headline)
                            .frame(maxWidth: .infinity)
                            .frame(height: 50)
                            .contentShape(Rectangle())
                    }
                    .buttonStyle(.borderedProminent)
                    .padding(.horizontal, 40)
                    .padding(.bottom, 20)
                }
            }
        }
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    showingDeploySheet = true
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .sheet(isPresented: $showingDeploySheet) {
            AccountCreationView(
                accountInfo: .init(
                    name: "Jane Doe",
                    userID: "jdoe@example.com",
                    domain: relyingPartyIdentifier
                )
            )
        }
    }
}

#Preview {
    NavigationStack {
        AccountsView(relyingPartyIdentifier: "soo-sdk-example-pages.pages.dev")
    }
}
