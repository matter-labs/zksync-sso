import Foundation

public struct UsageLimit: Codable, Equatable, Hashable, Sendable {
    public var limitType: LimitType
    public var limit: String
    public var period: String

    public init(limitType: LimitType, limit: String, period: String) {
        self.limitType = limitType
        self.limit = limit
        self.period = period
    }

    public static let unlimited = UsageLimit(limitType: .unlimited, limit: "0x0", period: "0x0")
    public static let zero = UsageLimit(limitType: .lifetime, limit: "0x0", period: "0x0")
}
