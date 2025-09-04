import Foundation

struct AccountSession {
    let accountDetails: AccountDetails
    let signers: AccountSigners

    init(accountDetails: AccountDetails, signers: AccountSigners) {
        self.accountDetails = accountDetails
        self.signers = signers
    }
}
