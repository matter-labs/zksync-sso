import SwiftUI
import ZKsyncSSO

struct ContentView: View {
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text(greetRust(name: "Rust"))
        }
        .padding()
    }
}

#Preview {
    ContentView()
}
