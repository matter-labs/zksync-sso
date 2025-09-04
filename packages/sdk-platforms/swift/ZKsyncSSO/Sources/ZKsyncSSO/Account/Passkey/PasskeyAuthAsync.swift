
import AuthenticationServices
import CryptoKit
import Foundation
@preconcurrency import ZKsyncSSOFFI

public final class PasskeyAuthAsync: PasskeyAuthenticatorAsync & Sendable {
    
    private let authenticator: PasskeyAsyncHelperAPI
    
    public init(authenticator: PasskeyAsyncHelperAPI) {
        self.authenticator = authenticator
    }
    
    public func signMessage(message: Data) async throws -> Data {
        try await authenticator.authenticate(message: message)
    }
}
