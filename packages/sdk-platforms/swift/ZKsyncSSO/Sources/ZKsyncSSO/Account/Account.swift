import Foundation

public struct Account: Sendable {
    public var address: String
    public var uniqueAccountId: String

    public init(address: String, uniqueAccountId: String) {
        self.address = address
        self.uniqueAccountId = uniqueAccountId
    }
}
