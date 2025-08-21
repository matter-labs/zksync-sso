import Foundation

public struct TransferSpec: Codable, Equatable, Hashable, Sendable {
    public var target: String
    public var maxValuePerUse: String
    public var valueLimit: UsageLimit

    public init(target: String, maxValuePerUse: String, valueLimit: UsageLimit) {
        self.target = target
        self.maxValuePerUse = maxValuePerUse
        self.valueLimit = valueLimit
    }
}
