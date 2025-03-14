import Foundation
import ZKsyncSSOFFI

public struct TransactionRequest {
    public var to: String
    public var value: String
    public var from: String
    
    public init(to: String, value: String, from: String) {
        self.to = to
        self.value = value
        self.from = from
    }
}

extension TransactionRequest {
    var inner: ZKsyncSSOFFI.Transaction {
        ZKsyncSSOFFI.Transaction(to: to, value: value, from: from)
    }
}
