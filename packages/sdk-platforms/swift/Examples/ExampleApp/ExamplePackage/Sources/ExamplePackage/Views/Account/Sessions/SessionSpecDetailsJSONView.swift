import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

#if os(iOS)
    import UIKit
#endif

struct SessionSpecDetailsJSONView: View {
    let sessionSpec: SessionSpec
    @State private var showCopiedToast = false

    private var sessionConfigJson: String {
        try! sessionSpec.toJsonString(pretty: true)
    }

    var body: some View {
        ZStack {
            VStack(alignment: .leading, spacing: 8) {
                Text("JSON Configuration")
                    .font(.subheadline)
                    .fontWeight(.medium)

                Text(sessionConfigJson)
                    .font(.system(.caption, design: .monospaced))
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(12)
                    // Use a semantic system background for code block look
                    #if os(iOS)
                        .background(Color(UIColor.systemGray6))
                    #else
                        .background(Color.gray.opacity(0.12))
                    #endif
                    .cornerRadius(8)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        #if os(iOS)
                            UIPasteboard.general.string = sessionConfigJson
                        #endif
                        withAnimation(.spring(response: 0.25, dampingFraction: 0.9)) {
                            showCopiedToast = true
                        }
                        DispatchQueue.main.asyncAfter(deadline: .now() + 1.2) {
                            withAnimation(.easeOut(duration: 0.2)) {
                                showCopiedToast = false
                            }
                        }
                    }
            }

            if showCopiedToast {
                ToastView(
                    icon: "doc.on.doc.fill",
                    iconColor: .blue,
                    message: "Session config copied"
                )
                .padding()
                .transition(.scale.combined(with: .opacity))
            }
        }
    }
}

#Preview {
    SessionSpecDetailsJSONView(
        sessionSpec: SessionSpec.default
    )
    .padding()
}
