import Foundation

public enum Status: String, Codable, Equatable, Hashable, Sendable {
    case notInitialized = "NotInitialized"
    case active = "Active"
    case closed = "Closed"
    
    public var isNotInitialized: Bool {
        self == .notInitialized
    }
    
    public var isActive: Bool {
        self == .active
    }
    
    public var isClosed: Bool {
        self == .closed
    }
}