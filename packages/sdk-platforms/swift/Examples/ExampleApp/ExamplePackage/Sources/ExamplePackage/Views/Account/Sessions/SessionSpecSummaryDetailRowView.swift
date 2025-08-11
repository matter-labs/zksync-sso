import SwiftUI

struct SessionSpecSummaryDetailRowView: View {
    let label: String
    let value: String

    var body: some View {
        HStack(spacing: 8) {
            Text(label + ":")
                .fontWeight(.medium)
                .foregroundStyle(.secondary)

            // Expanding container for the value that consumes remaining space
            HStack {
                Spacer()
                Text(value)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
            .frame(maxWidth: .infinity, alignment: .trailing)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

#Preview {
    VStack(alignment: .leading, spacing: 12) {
        SessionSpecSummaryDetailRowView(
            label: "Signer",
            value: "0x9BbC92a33DfeE0b40D16F7b3B1a55500d972479"
        )
        SessionSpecSummaryDetailRowView(
            label: "Expires At",
            value: "15 Aug 2025 at 13:18"
        )
        SessionSpecSummaryDetailRowView(
            label: "Fee Limit",
            value: "lifetime - 1000...000000000000"
        )
    }
    .padding()
}
