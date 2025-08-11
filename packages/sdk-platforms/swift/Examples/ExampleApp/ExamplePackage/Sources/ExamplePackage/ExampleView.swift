import SwiftUI
import ZKsyncSSO

public struct ExampleView: View {
    let relyingPartyIdentifier: String

    @State private var accountSession: AccountSession?

    public init(relyingPartyIdentifier: String, bundleIdentifier: String) {
        self.relyingPartyIdentifier = relyingPartyIdentifier

        ZKsyncSSO.initLogger(bundleIdentifier: bundleIdentifier, level: .trace)
    }

    public var body: some View {
        NavigationStack {
            if let session = accountSession {
                AccountDetailsView(
                    account: session.accountDetails,
                    signers: session.signers,
                    onLogout: {
                        accountSession = nil
                    }
                )
            } else {
                LoggedOutView(
                    accountInfo: AccountInfo(
                        name: "Jane Doe",
                        userID: "jdoe@example.com",
                        domain: relyingPartyIdentifier
                    ),
                    onAccountCreated: { session in
                        self.accountSession = session
                    },
                    onSignedIn: { session in
                        self.accountSession = session
                    }
                )
            }
        }
    }
}

#Preview {
    ExampleView(
        relyingPartyIdentifier: "auth-test.zksync.dev",
        bundleIdentifier: "io.jackpooley.MLSSOExample"
    )
}
