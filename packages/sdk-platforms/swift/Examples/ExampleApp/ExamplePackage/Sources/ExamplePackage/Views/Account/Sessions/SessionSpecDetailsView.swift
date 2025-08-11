import SwiftUI
import ZKsyncSSO

struct SessionSpecDetailsView: View {
    let sessionSpec: SessionSpec

    private var sessionConfigJson: String {
        try! sessionSpec.toJsonString(pretty: true)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Session Details")
                .font(.headline)

            SessionSpecSummaryView(sessionSpec: sessionSpec)

            SessionSpecDetailsJSONView(sessionSpec: sessionSpec)
        }
        .padding()
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

#Preview {
    ScrollView {
        SessionSpecDetailsView(
            sessionSpec: SessionSpec.default
        )
    }
}
