import Foundation

public struct SessionState: Codable, Equatable, Hashable, Sendable {
    public var status: Status
    public var feesRemaining: String
    public var transferValue: [LimitState]
    public var callValue: [LimitState]
    public var callParams: [LimitState]
    
    public init(
        status: Status,
        feesRemaining: String,
        transferValue: [LimitState],
        callValue: [LimitState],
        callParams: [LimitState]
    ) {
        self.status = status
        self.feesRemaining = feesRemaining
        self.transferValue = transferValue
        self.callValue = callValue
        self.callParams = callParams
    }
    
    public var isActive: Bool {
        status.isActive
    }
    
    public var isClosed: Bool {
        status.isClosed
    }
}