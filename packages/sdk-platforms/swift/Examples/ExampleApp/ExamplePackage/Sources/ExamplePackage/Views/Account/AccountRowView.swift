import SwiftUI

struct AccountRowView: View {
    let account: DeployedAccount
    let relyingPartyIdentifier: String
    
    var body: some View {
        NavigationLink {
            AccountDetailsView(
                address: account.address,
                uniqueAccountId: account.uniqueAccountId,
                relyingPartyIdentifier: relyingPartyIdentifier
            )
        } label: {
            VStack(alignment: .leading) {
                Text(account.name)
                    .font(.headline)
                
                Text(account.uniqueAccountId)
                    .font(.caption)
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .foregroundStyle(.secondary)
                    
                Text(account.address)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }
}

#Preview {
    NavigationStack {
        List {
            AccountRowView(
                account: .init(
                    info: .init(
                        name: "Jane Doe",
                        userID: "jdoe",
                        domain: "example.com"
                    ),
                    address: "0x5f14BCA4eA5F87f590cF77D0001312451602b6CD",
                    uniqueAccountId: "uniqueAccountId"
                ),
                relyingPartyIdentifier: "example.com"
            )
        }
    }
} 
