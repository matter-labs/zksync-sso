import SwiftUI
import ExamplePackageUIComponents

struct TransactionFormFields: View {
    let fromAddress: String
    @Binding var toAddress: String
    @Binding var amount: String

    var body: some View {
        Section("From") {
            AddressView(address: fromAddress)
                .listRowInsets(EdgeInsets())
                .listRowBackground(Color.clear)
        }

        Section("To") {
            HStack {
                TextField("Recipient Address", text: $toAddress)
                    .autocorrectionDisabled()
                    .textInputAutocapitalization(.never)
                    .font(.system(.body, design: .monospaced))

                if !toAddress.isEmpty {
                    Button {
                        withAnimation {
                            toAddress = ""
                        }
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.gray)
                            .padding(.leading, 8)
                    }
                }
            }
        }

        Section("Amount") {
            HStack {
                TextField("Amount in ETH", text: $amount)
                    .keyboardType(.decimalPad)

                if !amount.isEmpty {
                    Button {
                        withAnimation {
                            amount = ""
                        }
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.gray)
                            .padding(.leading, 8)
                    }
                }
            }
        }
    }
}

#Preview {
    Form {
        TransactionFormFields(
            fromAddress: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            toAddress: .constant("0x71C7656EC7ab88b098defB751B7401B5f6d8976F"),
            amount: .constant("1.5")
        )
    }
}
