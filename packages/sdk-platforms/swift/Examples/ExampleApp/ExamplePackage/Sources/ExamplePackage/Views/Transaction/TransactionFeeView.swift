import SwiftUI

struct TransactionFeeView: View {
    let fee: String
    let isPreparing: Bool

    var body: some View {
        Section("Transaction Fee") {
            HStack {
                Text(fee)
                    .font(.system(.body, design: .monospaced))
                    .foregroundStyle(.secondary)

                if isPreparing {
                    Spacer()
                    ProgressView()
                        .controlSize(.small)
                }
            }
        }
    }
}

#Preview("Normal") {
    Form {
        TransactionFeeView(
            fee: "0.000123 ETH",
            isPreparing: false
        )
    }
}

#Preview("Preparing") {
    Form {
        TransactionFeeView(
            fee: "0.000123 ETH",
            isPreparing: true
        )
    }
}
