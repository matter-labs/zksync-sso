import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

struct SendTransactionView: View {
    let fromAccount: AccountDetails
    @Environment(\.dismiss) private var dismiss

    @State private var toAddress: String = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
    @State private var amount: String = "1.0"
    @State private var isConfirming = false
    @State private var error: String?
    @State private var showingSuccess = false
    @State private var preparedTransaction: PreparedTransaction?
    @State private var isPreparing = false

    private let passkeyAuth: PasskeyAuthSync
    private let accountClient: AccountClient
    private let onTransactionSent: () -> Void

    init(
        fromAccount: AccountDetails,
        passkeyAuth: PasskeyAuthSync,
        onTransactionSent: @escaping () -> Void = {}
    ) {
        print("Initializing SendTransactionView")
        self.fromAccount = fromAccount
        self.passkeyAuth = passkeyAuth
        self.onTransactionSent = onTransactionSent
        self.accountClient = AccountClient(
            account: .init(address: fromAccount.address, uniqueAccountId: ""),
            authenticator: passkeyAuth
        )
    }

    var body: some View {
        NavigationView {
            Form {
                TransactionFormFields(
                    fromAddress: fromAccount.address,
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
                        Text(error)
                            .foregroundStyle(.red)
                    }
                }

                ActionButton(
                    title: "Send Transaction",
                    progressTitle: "Sending Transaction...",
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
            .passkeyPresentation(passkeyAuth.manager)
            .id("SendTransactionView")
            .onAppear { print("SendTransactionView appeared") }
            .navigationTitle("Send Transaction")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") {
                        dismiss()
                    }
                }
            }
        }
        .onChange(of: toAddress) { _ in
            prepareTransaction()
        }
        .onChange(of: amount) { _ in
            prepareTransaction()
        }
        .task {
            await prepareTransaction()
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

        guard let amountInEth = Double(amount) else { return }
        let amountInWei = String(Int(amountInEth * 1_000_000_000_000_000_000.0))

        Task {
            isPreparing = true
            defer { isPreparing = false }

            do {
                let from = fromAccount.address
                let transaction = TransactionRequest(
                    to: toAddress,
                    value: amountInWei,
                    from: from
                )

                let prepared = try await accountClient.prepareTransaction(
                    transaction: transaction
                )
                print(
                    "Prepared transaction JSON: \(prepared.transactionRequestJson)"
                )
                preparedTransaction = prepared
                error = nil
            } catch {
                self.error = error.localizedDescription
                preparedTransaction = nil
                print("Error preparing transaction: \(error)")
            }
        }
    }

    private func confirmTransaction() {
        guard let amountInEth = Double(amount) else { return }
        
        let amountInWei = String(Int(amountInEth * 1_000_000_000_000_000_000.0))

        isConfirming = true
        error = nil

        Task {
            do {
                try await accountClient.sendTransaction(
                    to: toAddress,
                    amount: amountInWei
                )

                withAnimation {
                    showingSuccess = true
                }

                onTransactionSent()

                try? await Task.sleep(for: .seconds(1.5))

                dismiss()
            } catch {
                self.error = error.localizedDescription
                print(error)
                isConfirming = false
            }
        }
    }
}

#Preview {
    SendTransactionView(
        fromAccount: AccountDetails(
            address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            uniqueAccountId: "uniqueAccountId",
            balance: "1000000000000000000"
        ),
        passkeyAuth: PasskeyAuthSync(
            authenticator: PasskeyAuthenticatorHelper(
                manager: PasskeyManager(
                    relyingPartyIdentifier: "example.app"
                )
            )
        )
    )
}
