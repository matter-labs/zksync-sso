import Foundation
@preconcurrency import ZKsyncSSOFFI

public struct TransactionRequest {
    public var to: String?
    public var value: String?
    public var input: String?
    
    public init(
        to: String? = nil,
        value: String? = nil,
        input: String? = nil
    ) {
        self.to = to
        self.value = value
        self.input = input
    }
}

extension ZKsyncSSOFFI.Transaction {
    static func from(
        request: TransactionRequest,
        account from: String
    ) -> Self {
        Self(
            from: from,
            to: request.to,
            value: request.value,
            input: request.input
        )
    }
}
