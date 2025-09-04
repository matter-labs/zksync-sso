import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

struct TransactionConfiguration {

    typealias PrepareTransactionFn = (String, String) async throws -> PreparedTransaction
    typealias ConfirmTransactionFn = (String, String) async throws -> Void

    let title: String
    let buttonTitle: String
    let buttonProgressTitle: String
    let initialToAddress: String
    let initialAmount: String
    let prepareTransaction: PrepareTransactionFn
    let confirmTransaction: ConfirmTransactionFn
}

struct SendTransactionView: View {
    let configuration: TransactionConfiguration
    let fromAddress: String

    @Environment(\.dismiss) private var dismiss

    @State private var toAddress: String
    @State private var amount: String
    @State private var isConfirming = false
    @State private var error: UIError?
    @State private var showingSuccess = false
    @State private var preparedTransaction: PreparedTransaction?
    @State private var isPreparing = false

    private let onTransactionSent: () -> Void

    init(
        configuration: TransactionConfiguration,
        fromAddress: String,
        onTransactionSent: @escaping () -> Void = {}
    ) {
        self.configuration = configuration
        self.fromAddress = fromAddress
        self.onTransactionSent = onTransactionSent
        self._toAddress = State(initialValue: configuration.initialToAddress)
        self._amount = State(initialValue: configuration.initialAmount)
    }

    var body: some View {
        NavigationView {
            Form {
                TransactionFormFields(
                    fromAddress: fromAddress,
                    toAddress: $toAddress,
                    amount: $amount
                )

                if let prepared = preparedTransaction {
                    TransactionFeeView(
                        fee: prepared.displayFee,
                        isPreparing: isPreparing
                    )
                }

                if let error = error {
                    Section {
                        Button {
                            #if os(iOS)
                                UIPasteboard.general.string = error.message
                            #elseif os(macOS)
                                NSPasteboard.general.clearContents()
                                NSPasteboard.general.setString(error.message, forType: .string)
                            #endif
                        } label: {
                            HStack {
                                Text(error.message)
                                    .foregroundStyle(.red)

                                Spacer()

                                Image(systemName: "doc.on.doc")
                                    .foregroundStyle(.secondary)
                                    .font(.system(size: 14))
                            }
                        }
                    }
                }

                ActionButton(
                    title: configuration.buttonTitle,
                    progressTitle: configuration.buttonProgressTitle,
                    icon: "paperplane.circle.fill",
                    isLoading: isConfirming || isPreparing,
                    isDisabled: !isValidInput || preparedTransaction == nil,
                    style: .prominent,
                    action: confirmTransaction
                )
            }
            .overlay {
                if showingSuccess {
                    ToastView(
                        icon: "checkmark.circle.fill",
                        iconColor: .green,
                        message: "Transaction Sent!"
                    )
                }
            }
            .navigationTitle(configuration.title)
            #if os(iOS)
                .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
        }
        .onChange(of: toAddress) { _, _ in
            prepareTransaction()
        }
        .onChange(of: amount) { _, _ in
            prepareTransaction()
        }
        .task {
            prepareTransaction()
        }
    }

    private var isValidInput: Bool {
        !toAddress.isEmpty && !amount.isEmpty && Double(amount) != nil && toAddress.hasPrefix("0x")
            && toAddress.count == 42
    }

    private func prepareTransaction() {
        guard isValidInput else {
            preparedTransaction = nil
            return
        }

        guard Double(amount) != nil else {
            return
        }

        Task {
            isPreparing = true
            defer { isPreparing = false }

            do {
                preparedTransaction = try await configuration.prepareTransaction(toAddress, amount)
                error = nil
            } catch {
                self.error = UIError(from: error)
                preparedTransaction = nil
            }
        }
    }

    private func confirmTransaction() {
        guard Double(amount) != nil else { return }

        isConfirming = true
        error = nil

        Task {
            do {
                try await configuration.confirmTransaction(toAddress, amount)

                withAnimation {
                    showingSuccess = true
                }

                onTransactionSent()

                try? await Task.sleep(for: .seconds(1.5))

                dismiss()
            } catch {
                self.error = UIError(from: error)
                isConfirming = false
            }
        }
    }
}

#Preview("Regular Transaction") {
    SendTransactionView(
        configuration: TransactionConfiguration(
            title: "Send Transaction",
            buttonTitle: "Send Transaction",
            buttonProgressTitle: "Sending Transaction...",
            initialToAddress: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            initialAmount: "1.0",
            prepareTransaction: { _, _ in
                PreparedTransaction(
                    transactionRequestJson: "{}",
                    from: "",
                    to: "",
                    value: "",
                    displayFee: "0.001 ETH",
                )
            },
            confirmTransaction: { _, _ in
                // Mock implementation
            }
        ),
        fromAddress: "0x742d35Cc6634C0532925a3b844Bc9e7595f62411"
    )
}

#Preview("Session Transaction") {
    SendTransactionView(
        configuration: TransactionConfiguration(
            title: "Send Session Transaction",
            buttonTitle: "Send Transaction",
            buttonProgressTitle: "Sending Transaction...",
            initialToAddress: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            initialAmount: "0.5",
            prepareTransaction: { _, _ in
                PreparedTransaction(
                    transactionRequestJson: "{}",
                    from: "",
                    to: "",
                    value: "",
                    displayFee: "0.001 ETH",
                )
            },
            confirmTransaction: { _, _ in
                // Mock implementation
            }
        ),
        fromAddress: "0x742d35Cc6634C0532925a3b844Bc9e7595f62411"
    )
}
