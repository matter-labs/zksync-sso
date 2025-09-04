import Foundation

public enum LimitType: String, Codable, Hashable, Sendable {
    case unlimited = "Unlimited"
    case lifetime = "Lifetime"
    case allowance = "Allowance"
}
