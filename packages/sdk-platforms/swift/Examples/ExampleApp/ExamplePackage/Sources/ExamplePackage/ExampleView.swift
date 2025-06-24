import SwiftUI
import ZKsyncSSO

public struct ExampleView: View {
    
    let relyingPartyIdentifier: String
    
    @State private var accountDetails: AccountDetails?

    public init(relyingPartyIdentifier: String, bundleIdentifier: String) {
        self.relyingPartyIdentifier = relyingPartyIdentifier
        
        ZKsyncSSO.initLogger(bundleIdentifier: bundleIdentifier, level: .trace)
    }

    public var body: some View {
        NavigationStack {
            if let account = accountDetails {
                AccountDetailsView(
                    account: account,
                    onLogout: {
                        accountDetails = nil
                    }
                )
            } else {
                LoggedOutView(
                    accountInfo: AccountInfo(
                        name: "Jane Doe",
                        userID: "jdoe@example.com",
                        domain: relyingPartyIdentifier
                    ),
                    onAccountCreated: { account in
                        self.accountDetails = account
                    },
                    onSignedIn: { account in
                        self.accountDetails = account
                    }
                )
            }
        }
    }
}

#Preview {
    ExampleView(
        relyingPartyIdentifier: "soo-sdk-example-pages.pages.dev",
        bundleIdentifier: "io.jackpooley.MLSSOExample"
    )
}
