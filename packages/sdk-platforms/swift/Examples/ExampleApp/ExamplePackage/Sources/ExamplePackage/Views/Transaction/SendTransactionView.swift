import ExamplePackageUIComponents
import SwiftUI
import ZKsyncSSO

struct SendTransactionView: View {
    let fromAccount: DeployedAccount
    @Environment(\.dismiss) private var dismiss

    @State private var toAddress: String = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
    @State private var amount: String = "1.0"
    @State private var isConfirming = false
    @State private var error: String?
    @State private var showingSuccess = false
    @State private var preparedTransaction: PreparedTransaction?
    @State private var isPreparing = false
    
    @Environment(\.authorizationController) private var authorizationController
    
    private let onTransactionSent: () -> Void

    init(
        fromAccount: DeployedAccount,
        onTransactionSent: @escaping () -> Void = {}
    ) {
        print("Initializing SendTransactionView")
        self.fromAccount = fromAccount
        self.onTransactionSent = onTransactionSent
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
        
        let authenticator = PasskeyAuthenticatorHelper(
            controllerProvider: { self.authorizationController },
            relyingPartyIdentifier: "soo-sdk-example-pages.pages.dev"
        )
        
        let accountClient = AccountClient(
            account: .init(
                address: fromAccount.address,
                uniqueAccountId: fromAccount.uniqueAccountId
            ),
            authenticatorAsync: PasskeyAuthAsync(
                authenticator: authenticator
            )
        )

        Task {
            isPreparing = true
            defer { isPreparing = false }

            do {
                let from = fromAccount.address
                let transaction = TransactionRequest(
                    from: from,
                    to: toAddress,
                    value: amountInWei
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
        
        let authenticator = PasskeyAuthenticatorHelper(
            controllerProvider: { self.authorizationController },
            relyingPartyIdentifier: "soo-sdk-example-pages.pages.dev"
        )
        
        let accountClient = AccountClient(
            account: .init(
                address: fromAccount.address,
                uniqueAccountId: fromAccount.uniqueAccountId
            ),
            authenticatorAsync: PasskeyAuthAsync(
                authenticator: authenticator
            )
        )

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
        fromAccount: .init(
            info: .init(
                name: "Jane Doe",
                userID: "jdoe@example.com",
                domain: "example.com"
            ),
            address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            uniqueAccountId: "jdoe@example.com"
        )
    )
}
