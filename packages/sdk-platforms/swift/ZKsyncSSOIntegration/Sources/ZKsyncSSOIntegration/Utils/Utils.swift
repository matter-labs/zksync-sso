import Foundation
import ZKsyncSSO

// MARK: - General Utility Functions

// Note: Session state validation functions have been moved to SessionStateValidation.swift
// This file is reserved for other general utility functions.

// MARK: - Session Key Validation

/// Verifies the session key matches expected bytes (for testing)
/// - Parameters:
///   - sessionKeyHex: The session key in hex format
///   - expectedBytes: The expected byte array
/// - Throws: Error if validation fails
public func validateSessionKey(sessionKeyHex: String, expectedBytes: [UInt8]) throws {
    guard let sessionKeyData = Data(hex: String(sessionKeyHex.dropFirst(2))) else {
        throw NSError(
            domain: "SessionValidationError",
            code: 1,
            userInfo: [NSLocalizedDescriptionKey: "Failed to parse session key hex"]
        )
    }
    
    let sessionKeyBytes = Array(sessionKeyData)
    
    print("🔑 Session key validation:")
    print("  Session key bytes: \(sessionKeyBytes)")
    print("  Expected bytes: \(expectedBytes)")
    
    guard sessionKeyBytes == expectedBytes else {
        throw NSError(
            domain: "SessionValidationError",
            code: 2,
            userInfo: [
                NSLocalizedDescriptionKey: "Session key bytes should match expected value from test"
            ]
        )
    }
    
    print("✅ Session key bytes match expected value")
}

// MARK: - Account Ownership Validation

/// Validates that an address is a K1 owner of an account
/// - Parameters:
///   - accountAddress: The account address to check
///   - ownerAddress: The address to verify as owner
///   - config: The configuration to use (defaults to Config.default)
///   - description: Optional description for logging
/// - Throws: Error if validation fails
public func validateIsK1Owner(
    accountAddress: String,
    ownerAddress: String,
    config: Config = .default,
    description: String = "K1 owner validation"
) async throws {
    print("\n🔍 \(description)")
    print("  Account: \(accountAddress)")
    print("  Owner address: \(ownerAddress)")
    
    let isOwnerArgs = IsK1OwnerArgs(
        account: accountAddress,
        ownerAddress: ownerAddress
    )
    
    let isValidOwner = try await isK1Owner(
        args: isOwnerArgs,
        config: config
    )
    
    print("  Validation result: \(isValidOwner)")
    
    guard isValidOwner else {
        throw NSError(
            domain: "AccountValidationError",
            code: 1,
            userInfo: [
                NSLocalizedDescriptionKey: "Address \(ownerAddress) is not a valid K1 owner of account \(accountAddress)"
            ]
        )
    }
    
    print("✅ K1 owner validation successful")
}

// MARK: - Data Extensions

extension Data {
    init?(hex: String) {
        let len = hex.count / 2
        var data = Data(capacity: len)
        var i = hex.startIndex
        for _ in 0..<len {
            let j = hex.index(i, offsetBy: 2)
            let bytes = hex[i..<j]
            if var num = UInt8(bytes, radix: 16) {
                data.append(&num, count: 1)
            } else {
                return nil
            }
            i = j
        }
        self = data
    }
}