import Foundation
import ZKsyncSSOFFI

public struct PreparedTransaction: Sendable {
    public var transactionRequestJson: String
    public var from: String
    public var to: String
    public var value: String
    public var displayFee: String
    
    public init(transactionRequestJson: String, from: String, to: String, value: String, displayFee: String) {
        self.transactionRequestJson = transactionRequestJson
        self.from = from
        self.to = to
        self.value = value
        self.displayFee = displayFee
    }
}

extension ZKsyncSSOFFI.PreparedTransaction {
    var wrappedValue: ZKsyncSSO.PreparedTransaction {
        .init(
            transactionRequestJson: transactionRequestJson,
            from: from,
            to: to,
            value: value,
            displayFee: displayFee
        )
    }
}
