import SwiftUI
import ExamplePackage

struct ContentView: View {
    var body: some View {
        ExampleView(
            relyingPartyIdentifier: "auth-test.zksync.dev",
            bundleIdentifier: "dev.zksync.auth-test.SSOExample"
        )
    }
}

#Preview {
    ContentView()
}
