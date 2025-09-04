import Foundation

public enum Condition: String, Codable, Hashable, Sendable {
    case unconstrained = "Unconstrained"
    case equal = "Equal"
    case greater = "Greater"
    case less = "Less"
    case greaterEqual = "GreaterEqual"
    case lessEqual = "LessEqual"
    case notEqual = "NotEqual"
}
