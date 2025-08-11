import SwiftUI
import ZKsyncSSO

struct SessionsView: View {
    let account: DeployedAccount
    let signers: AccountSigners

    @EnvironmentObject private var sessionsStore: SessionsStore
    @State private var showingCreateSession = false

    private var sessions: [Session] {
        sessionsStore.getSessions(for: account.address)
    }

    var body: some View {
        NavigationStack {
            Group {
                if sessions.isEmpty {
                    SessionsEmptyView(
                        onCreateTapped: { showingCreateSession = true }
                    )
                } else {
                    SessionsListView(
                        sessions: sessions,
                        account: account,
                        signers: signers,
                        onSelect: { _ in }
                    )
                }
            }
            .navigationTitle("Sessions")
            #if os(iOS)
                .toolbar {
                    ToolbarItem(placement: .navigationBarTrailing) {
                        Button {
                            showingCreateSession = true
                        } label: {
                            Image(systemName: "plus")
                        }
                    }
                }
            #endif
            .task {
                // TODO: refresh sessions
            }
            .sheet(isPresented: $showingCreateSession) {
                SessionCreationView(
                    accountAddress: account.address,
                    signers: signers,
                    onSessionCreated: { newSession in
                        sessionsStore.addSession(newSession, for: account.address)
                        showingCreateSession = false
                    }
                )
            }
        }
    }
}

#Preview("Empty State") {
    SessionsView(
        account: .init(
            info: .init(
                name: "Jane Doe", userID: "jdoe@example.com",
                domain: "auth-test.zksync.dev"
            ),
            address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            uniqueAccountId: "jdoe@example.com"
        ),
        signers: .default
    )
    .environmentObject(SessionsStore.shared)
}

#Preview("1 Session") {
    let store = SessionsStore.preview()
    let account = DeployedAccount(
        info: .init(
            name: "Jane Doe", userID: "jdoe@example.com",
            domain: "auth-test.zksync.dev"
        ),
        address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        uniqueAccountId: "jdoe@example.com"
    )

    store.addSession(
        Session(
            createdAt: Date(),
            sessionSpec: SessionSpec.default
        ),
        for: account.address
    )

    return SessionsView(
        account: account,
        signers: .default
    )
    .environmentObject(store)
}

#Preview("5 Sessions") {
    let store = SessionsStore.preview()
    let account = DeployedAccount(
        info: .init(
            name: "Jane Doe", userID: "jdoe@example.com",
            domain: "auth-test.zksync.dev"
        ),
        address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        uniqueAccountId: "jdoe@example.com"
    )

    let sessions = [
        // Session 1: Created 2 days ago, expires in 7 days from creation (5 days from now)
        Session(
            createdAt: Date().addingTimeInterval(-86400 * 2),
            sessionSpec: SessionSpec.default.with(
                expiry: Date().addingTimeInterval(-86400 * 2 + 86400 * 7)
            )
        ),
        // Session 2: Created 1 day ago, expires in 5 days from creation (4 days from now)
        Session(
            createdAt: Date().addingTimeInterval(-86400),
            sessionSpec: SessionSpec.default.with(
                expiry: Date().addingTimeInterval(-86400 + 86400 * 5)
            )
        ),
        // Session 3: Created 12 hours ago, expires in 3 days from creation (2.5 days from now)
        Session(
            createdAt: Date().addingTimeInterval(-43200),
            sessionSpec: SessionSpec.default.with(
                expiry: Date().addingTimeInterval(-43200 + 86400 * 3)
            )
        ),
        // Session 4: Created 6 hours ago, expires in 2 days from creation (1.75 days from now)
        Session(
            createdAt: Date().addingTimeInterval(-21600),
            sessionSpec: SessionSpec.default.with(
                expiry: Date().addingTimeInterval(-21600 + 86400 * 2)
            )
        ),
        // Session 5: Created 1 hour ago, expires in 1 day from creation (23 hours from now)
        Session(
            createdAt: Date().addingTimeInterval(-3600),
            sessionSpec: SessionSpec.default.with(
                expiry: Date().addingTimeInterval(-3600 + 86400)
            )
        ),
    ]

    for session in sessions {
        store.addSession(session, for: account.address)
    }

    return SessionsView(
        account: account,
        signers: .default
    )
    .environmentObject(store)
}
