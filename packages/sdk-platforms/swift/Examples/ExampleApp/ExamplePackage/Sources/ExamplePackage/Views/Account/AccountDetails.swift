import Foundation

struct AccountDetails {
    let address: String
    let uniqueAccountId: String
    var balance: String?
}

extension AccountDetails {
    
    var explorerURL: URL {
        URL(string: "https://explorer.zksync.io/address/\(self)")!
    }
}
