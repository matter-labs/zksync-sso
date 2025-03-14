import SwiftUI

public struct ExampleView: View {
    
    let relyingPartyIdentifier: String
    
    public init(relyingPartyIdentifier: String) {
        self.relyingPartyIdentifier = relyingPartyIdentifier
    }

    public var body: some View {
        NavigationStack {
            AccountsView(relyingPartyIdentifier: relyingPartyIdentifier)
        }
    }
}

#Preview {
    ExampleView(relyingPartyIdentifier: "soo-sdk-example-pages.pages.dev")
}
