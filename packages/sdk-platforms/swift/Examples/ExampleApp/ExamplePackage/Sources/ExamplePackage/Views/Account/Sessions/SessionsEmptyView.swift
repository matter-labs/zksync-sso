import SwiftUI
import ZKsyncSSO

struct SessionsEmptyView: View {
    let onCreateTapped: () -> Void

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "rectangle.stack.badge.plus")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)

            Text("No Sessions Yet")
                .font(.title3)

            Text(
                "Create a session to authorize limited actions without re-authenticating each time."
            )
            .font(.subheadline)
            .foregroundStyle(.secondary)
            .multilineTextAlignment(.center)
            .padding(.horizontal)

            Button(action: onCreateTapped) {
                Label("Create Session", systemImage: "plus")
            }
            .buttonStyle(.borderedProminent)
            .padding(.top, 8)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding()
    }
}

#Preview {
    SessionsEmptyView(onCreateTapped: {})
}
