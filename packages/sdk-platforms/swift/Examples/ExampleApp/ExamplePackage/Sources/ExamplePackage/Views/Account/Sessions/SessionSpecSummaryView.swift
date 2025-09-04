import SwiftUI
import ZKsyncSSO

struct SessionSpecSummaryView: View {
    let sessionSpec: SessionSpec

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            SessionSpecSummaryDetailRowView(label: "Signer", value: sessionSpec.signer)
            SessionSpecSummaryDetailRowView(
                label: "Expires At", value: formatTimestamp(sessionSpec.expiresAt)
            )
            SessionSpecSummaryDetailRowView(
                label: "Fee Limit",
                value: "\(sessionSpec.feeLimit.limitType) - \(sessionSpec.feeLimit.limit)"
            )
            SessionSpecSummaryDetailRowView(
                label: "Call Policies", value: "\(sessionSpec.callPolicies.count) policies"
            )
            SessionSpecSummaryDetailRowView(
                label: "Transfer Policies", value: "\(sessionSpec.transferPolicies.count) policies"
            )
        }
    }

    private func formatTimestamp(_ timestamp: String) -> String {
        guard let timeInterval = TimeInterval(timestamp) else {
            return timestamp
        }
        let date = Date(timeIntervalSince1970: timeInterval)
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short
        return formatter.string(from: date)
    }
}

#Preview {
    Form {
        Section(header: Text("Session Spec")) {
            SessionSpecSummaryView(
                sessionSpec: SessionSpec.default
            )
        }
    }
}
