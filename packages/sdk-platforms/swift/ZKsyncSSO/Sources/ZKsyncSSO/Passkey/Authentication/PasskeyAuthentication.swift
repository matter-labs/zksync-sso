import Foundation
import AuthenticationServices
import SwiftUI

@Sendable func performPasskeyAuthorizationRequest(
    challenge: Data,
    relyingPartyIdentifier: String,
    controller: AuthorizationController
) async throws -> AuthorizationPlatformPublicKeyCredentialAssertion {
    let assertion = try await performASPasskeyAuthorizationRequest(
        challenge: challenge,
        relyingPartyIdentifier: relyingPartyIdentifier,
        controller: controller
    )
    return AuthorizationPlatformPublicKeyCredentialAssertion.from(
        assertion: assertion
    )
}

private func performASPasskeyAuthorizationRequest(
    challenge: Data,
    relyingPartyIdentifier: String,
    controller: AuthorizationController
) async throws -> ASAuthorizationPlatformPublicKeyCredentialAssertion {
    let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(
        relyingPartyIdentifier: relyingPartyIdentifier
    )
    let assertionRequest = provider.createCredentialAssertionRequest(
        challenge: challenge
    )
    let assertion = try await controller.performRequest(assertionRequest)
    guard case .passkeyAssertion(let passkeyAssertion) = assertion else {
        fatalError("Unexpected authorization result: \(assertion)")
    }
    return passkeyAssertion
}

