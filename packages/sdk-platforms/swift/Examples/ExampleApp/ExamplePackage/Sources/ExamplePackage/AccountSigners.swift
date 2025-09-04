import Foundation

struct AccountSigners {
    var accountOwner: EOASigner
    var sessionOwner: EOASigner
    
    init(accountOwner: EOASigner, sessionOwner: EOASigner) {
        self.accountOwner = accountOwner
        self.sessionOwner = sessionOwner
    }
    
    static var `default`: AccountSigners {
        return AccountSigners(
            accountOwner: .accountOwner,
            sessionOwner: .sessionOwner
        )
    }
}