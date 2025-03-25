import Foundation
import AuthenticationServices
import SwiftUI

public protocol PasskeyAsyncHelperAPI: Sendable {
    func authenticate(message: Data) async throws -> Data
}

@MainActor
public struct PasskeyAuthenticatorHelper: PasskeyAsyncHelperAPI {
    
    public typealias ControllerProvider = () -> AuthorizationController
    
    public var controller: AuthorizationController {
        controllerProvider()
    }
    
    private let controllerProvider: ControllerProvider
    private let relyingPartyIdentifier: String
    
    public init(
        controllerProvider: @escaping ControllerProvider,
        relyingPartyIdentifier: String
    ) {
        self.controllerProvider = controllerProvider
        self.relyingPartyIdentifier = relyingPartyIdentifier
    }
    
    public func authenticate(message: Data) async throws -> Data {
        let assertion = try await performPasskeyAuthorizationRequest(
            challenge: message,
            relyingPartyIdentifier: relyingPartyIdentifier,
            controller: controller
        )
        
        print("Got assertion: \(assertion)")
        
        let authAssertionData = try JSONEncoder().encode(assertion)
        
        print(String(data: authAssertionData, encoding: .utf8) ?? "Couldn't decode assertion data")
        
        return authAssertionData
    }
}
