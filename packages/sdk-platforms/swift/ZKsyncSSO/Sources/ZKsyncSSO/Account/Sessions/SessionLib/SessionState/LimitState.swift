import Foundation

public struct LimitState: Codable, Equatable, Hashable, Sendable {
    public var remaining: String
    public var target: String
    public var selector: String
    public var index: String
    
    public init(
        remaining: String,
        target: String,
        selector: String,
        index: String
    ) {
        self.remaining = remaining
        self.target = target
        self.selector = selector
        self.index = index
    }
}