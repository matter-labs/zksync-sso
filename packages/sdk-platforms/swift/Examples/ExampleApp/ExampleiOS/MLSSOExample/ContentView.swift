import SwiftUI
import ExamplePackage

struct ContentView: View {
    var body: some View {
        ExampleView(
            relyingPartyIdentifier: "soo-sdk-example-pages.pages.dev",
            bundleIdentifier: "io.jackpooley.MLSSOExample"
        )
    }
}

#Preview {
    ContentView()
}
