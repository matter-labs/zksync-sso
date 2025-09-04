import Foundation
import ZKsyncSSO

// MARK: - Session State Validation System

/// Error types for session state validation failures
public enum SessionValidationError: Error, LocalizedError {
    case status(description: String)
    case feesRemaining(description: String)
    case transferValue(description: String)
    case callValue(description: String)
    case callParams(description: String)
    case custom(description: String)
    
    public var errorDescription: String? {
        switch self {
        case .status(let description):
            return "Status validation failed: \(description)"
        case .feesRemaining(let description):
            return "Fees remaining validation failed: \(description)"
        case .transferValue(let description):
            return "Transfer value validation failed: \(description)"
        case .callValue(let description):
            return "Call value validation failed: \(description)"
        case .callParams(let description):
            return "Call params validation failed: \(description)"
        case .custom(let description):
            return "Validation failed: \(description)"
        }
    }
}

/// Type alias for session state validation predicates
public typealias SessionStatePredicate = (SessionState) -> Result<Void, SessionValidationError>

// MARK: - Static Predicate Constructors

/// Validates that the session is active
public func sessionStateActive() -> SessionStatePredicate {
    return { state in
        state.isActive 
            ? .success(())
            : .failure(.status(description: "should be active but was \(state.status)"))
    }
}

/// Validates that the session is closed
public func sessionStateClosed() -> SessionStatePredicate {
    return { state in
        state.isClosed
            ? .success(())
            : .failure(.status(description: "should be closed but was \(state.status)"))
    }
}

/// Validates that the session has a specific status
public func sessionStateStatus(_ expectedStatus: Status) -> SessionStatePredicate {
    return { state in
        state.status == expectedStatus
            ? .success(())
            : .failure(.status(description: "should be \(expectedStatus) but was \(state.status)"))
    }
}

/// Validates fees remaining with a custom predicate
public func sessionStateFeesRemaining(_ pred: @escaping (String) -> Bool, description: String) -> SessionStatePredicate {
    return { state in
        pred(state.feesRemaining)
            ? .success(())
            : .failure(.feesRemaining(description: "\(description) (actual: \(state.feesRemaining))"))
    }
}

/// Validates that fees remaining equals a specific value
public func sessionStateFeesRemainingEquals(_ expected: String) -> SessionStatePredicate {
    return sessionStateFeesRemaining({ $0 == expected }, description: "should equal \(expected)")
}

/// Validates that fees remaining is greater than a value
public func sessionStateFeesRemainingGreaterThan(_ threshold: String) -> SessionStatePredicate {
    return sessionStateFeesRemaining(
        { (Int($0) ?? 0) > (Int(threshold) ?? 0) },
        description: "should be greater than \(threshold)"
    )
}

/// Validates that fees have been consumed (less than initial)
public func sessionStateFeesConsumed(from initial: String) -> SessionStatePredicate {
    return sessionStateFeesRemaining(
        { (Int($0) ?? 0) < (Int(initial) ?? 0) },
        description: "should be less than initial \(initial) (fees consumed)"
    )
}

/// Validates the number of transfer values
public func sessionStateTransferValueCount(_ expectedCount: Int) -> SessionStatePredicate {
    return { state in
        state.transferValue.count == expectedCount
            ? .success(())
            : .failure(.transferValue(description: "should have \(expectedCount) items but had \(state.transferValue.count)"))
    }
}

/// Validates transfer values with a custom predicate
public func sessionStateTransferValues(_ pred: @escaping ([LimitState]) -> Bool, description: String) -> SessionStatePredicate {
    return { state in
        pred(state.transferValue)
            ? .success(())
            : .failure(.transferValue(description: description))
    }
}

/// Validates a specific transfer value at an index
public func sessionStateTransferValueAt(_ index: Int, remaining: String? = nil, target: String? = nil) -> SessionStatePredicate {
    return { state in
        guard index < state.transferValue.count else {
            return .failure(.transferValue(description: "should have item at index \(index) but only has \(state.transferValue.count) items"))
        }
        
        let transfer = state.transferValue[index]
        
        if let expectedRemaining = remaining, transfer.remaining != expectedRemaining {
            return .failure(.transferValue(description: "item[\(index)].remaining should be \(expectedRemaining) but was \(transfer.remaining)"))
        }
        
        if let expectedTarget = target, transfer.target != expectedTarget {
            return .failure(.transferValue(description: "item[\(index)].target should be \(expectedTarget) but was \(transfer.target)"))
        }
        
        return .success(())
    }
}

/// Validates the number of call values
public func sessionStateCallValueCount(_ expectedCount: Int) -> SessionStatePredicate {
    return { state in
        state.callValue.count == expectedCount
            ? .success(())
            : .failure(.callValue(description: "should have \(expectedCount) items but had \(state.callValue.count)"))
    }
}

/// Validates the number of call params
public func sessionStateCallParamsCount(_ expectedCount: Int) -> SessionStatePredicate {
    return { state in
        state.callParams.count == expectedCount
            ? .success(())
            : .failure(.callParams(description: "should have \(expectedCount) items but had \(state.callParams.count)"))
    }
}

/// Custom validation with a description
public func sessionStateCustom(_ description: String, _ pred: @escaping (SessionState) -> Bool) -> SessionStatePredicate {
    return { state in
        pred(state)
            ? .success(())
            : .failure(.custom(description: description))
    }
}

// MARK: - Main Validation Functions

/// Validates session state by fetching it and applying predicates
/// - Parameters:
///   - accountAddress: The account address to fetch session state for
///   - sessionConfig: The session configuration JSON
///   - predicates: Array of validation predicates to apply
///   - config: The configuration to use (defaults to Config.default)
///   - description: Optional description for logging
/// - Returns: The fetched SessionState (discardable)
/// - Throws: The first SessionValidationError encountered
@discardableResult
public func validateSessionState(
    accountAddress: String,
    sessionConfig: String,
    predicates: [SessionStatePredicate],
    config: Config = .default,
    description: String = "Session state validation"
) async throws -> SessionState {
    print("\n🔍 Fetching session state for validation: \(description)")
    print("  Account: \(accountAddress)")
    
    let sessionStateArgs = GetSessionStateArgs(
        account: accountAddress,
        sessionConfig: sessionConfig
    )
    
    let sessionState = try await getSessionState(
        args: sessionStateArgs,
        config: config
    )
    
    try validateSessionStateImpl(
        actual: sessionState,
        predicates: predicates,
        description: description
    )
    
    return sessionState
}

/// Internal implementation that validates a session state against predicates
/// - Parameters:
///   - actual: The actual session state to validate
///   - predicates: Array of validation predicates to apply
///   - description: Optional description for logging
/// - Throws: The first SessionValidationError encountered
internal func validateSessionStateImpl(
    actual: SessionState,
    predicates: [SessionStatePredicate],
    description: String = "Session state validation"
) throws {
    print("\n🔍 \(description)")
    print("  Status: \(actual.status)")
    print("  Fees remaining: \(actual.feesRemaining)")
    print("  Transfer values: \(actual.transferValue.count)")
    print("  Call values: \(actual.callValue.count)")
    print("  Call params: \(actual.callParams.count)")
    
    for (index, predicate) in predicates.enumerated() {
        switch predicate(actual) {
        case .success:
            continue
        case .failure(let error):
            print("❌ Validation \(index + 1) failed: \(error.localizedDescription)")
            throw error
        }
    }
    
    print("✅ All \(predicates.count) validations passed")
}

// MARK: - Helper Functions

/// Helper function to validate that fees have been consumed
/// - Parameters:
///   - initialFees: The initial fee amount as a string
///   - remainingFees: The remaining fee amount as a string
///   - description: Optional description for the validation
/// - Returns: The amount of fees consumed
/// - Throws: Error if validation fails
public func validateFeesConsumed(
    initialFees: String,
    remainingFees: String,
    description: String = "Fees consumption validation"
) throws -> Int {
    let initial = Int(initialFees) ?? 0
    let remaining = Int(remainingFees) ?? 0
    let consumed = initial - remaining
    
    print("💰 \(description):")
    print("  Initial fees: \(initial) wei")
    print("  Remaining fees: \(remaining) wei")
    print("  Fees consumed: \(consumed) wei")
    
    guard consumed > 0 else {
        throw SessionValidationError.custom(
            description: "No fees were consumed. Initial: \(initial), Remaining: \(remaining)"
        )
    }
    
    return consumed
}