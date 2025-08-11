import Foundation

public struct CallSpec: Codable, Equatable, Hashable, Sendable {
    public var target: String
    public var selector: String
    public var maxValuePerUse: String
    public var valueLimit: UsageLimit
    public var constraints: [Constraint]

    public init(
        target: String,
        selector: String,
        maxValuePerUse: String,
        valueLimit: UsageLimit,
        constraints: [Constraint]
    ) {
        self.target = target
        self.selector = selector
        self.maxValuePerUse = maxValuePerUse
        self.valueLimit = valueLimit
        self.constraints = constraints
    }
}
