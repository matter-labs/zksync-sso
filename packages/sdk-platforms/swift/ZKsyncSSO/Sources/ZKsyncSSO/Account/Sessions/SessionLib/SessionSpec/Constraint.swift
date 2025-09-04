import Foundation

public struct Constraint: Codable, Equatable, Hashable, Sendable {
    public var condition: Condition
    public var index: UInt64
    public var refValue: String
    public var limit: UsageLimit

    public init(condition: Condition, index: UInt64, refValue: String, limit: UsageLimit) {
        self.condition = condition
        self.index = index
        self.refValue = refValue
        self.limit = limit
    }
}
