import Foundation

struct AccountDetails {
    let account: DeployedAccount
    
    var address: String {
        account.address
    }
    
    var uniqueAccountId: String {
        account.uniqueAccountId
    }
    
    var balance: String?
    
    init(account: DeployedAccount, balance: String? = nil) {
        self.account = account
        self.balance = balance
    }
}

extension AccountDetails {
    
    var explorerURL: URL {
        URL(string: "https://explorer.zksync.io/address/\(self)")!
    }
}
