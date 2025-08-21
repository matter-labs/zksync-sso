import SwiftUI
import ZKsyncSSO
import ZKsyncSSOIntegration

struct SessionsListView: View {
    let sessions: [Session]
    let account: DeployedAccountDetails
    let signers: AccountSigners
    let onSelect: (Session) -> Void

    var body: some View {
        List {
            Section(header: Text("Active Sessions")) {
                ForEach(sessions) { session in
                    NavigationLink(value: session) {
                        HStack {
                            VStack(alignment: .leading, spacing: 4) {
                                Text(shortHash(session.sessionHash))
                                    .font(.headline)
                                    .lineLimit(1)
                                    .truncationMode(.middle)
                                Text("Created \(format(date: session.createdAt))")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
            }
        }
        .navigationDestination(for: Session.self) { session in
            SessionDetailView(
                session: session,
                account: account,
                signers: signers
            )
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
    SessionsListView(
        sessions: [
            .init(
                createdAt: Date(),
                sessionSpec: SessionSpec.default,
                sessionKey: AccountSigners.default.sessionOwner.privateKeyHex
            )
        ],
        account: .default,
        signers: .default,
        onSelect: { _ in }
    )
}
