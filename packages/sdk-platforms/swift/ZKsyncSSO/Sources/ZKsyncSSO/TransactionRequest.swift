import Foundation
@preconcurrency import ZKsyncSSOFFI

public struct TransactionRequest {
    public var from: String
    public var to: String?
    public var value: String?
    public var input: String?
    
    public init(
        from: String,
        to: String? = nil,
        value: String? = nil,
        input: String? = nil
    ) {
        self.from = from
        self.to = to
        self.value = value
        self.input = input
    }
}

extension TransactionRequest {
    var inner: ZKsyncSSOFFI.Transaction {
        ZKsyncSSOFFI.Transaction(
            from: from,
            to: to,
            value: value,
            input: input
        )
    }
}
