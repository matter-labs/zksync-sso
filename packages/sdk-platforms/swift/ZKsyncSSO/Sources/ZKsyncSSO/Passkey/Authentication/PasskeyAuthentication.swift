import AuthenticationServices
import Foundation
import SwiftUI

@Sendable func performPasskeyAuthorizationRequest(
    challenge: Data,
    relyingPartyIdentifier: String,
    controller: AuthorizationController
) async throws -> AuthorizationPlatformPublicKeyCredentialAssertion {
    print("performPasskeyAuthorizationRequest")
    print(" - challenge: \(challenge)")
    print(" - relyingPartyIdentifier: \(relyingPartyIdentifier)")
    print(" - controller: \(controller)")
    let assertion = try await performASPasskeyAuthorizationRequest(
        challenge: challenge,
        relyingPartyIdentifier: relyingPartyIdentifier,
        controller: controller
    )
    print("performPasskeyAuthorizationRequest assertion: \(assertion)")
    return AuthorizationPlatformPublicKeyCredentialAssertion.from(
        assertion: assertion
    )
}

@MainActor
private func performASPasskeyAuthorizationRequest(
    challenge: Data,
    relyingPartyIdentifier: String,
    controller: AuthorizationController
) async throws -> ASAuthorizationPlatformPublicKeyCredentialAssertion {
    print("performASPasskeyAuthorizationRequest")
    let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(
        relyingPartyIdentifier: relyingPartyIdentifier
    )
    print("performASPasskeyAuthorizationRequest provider: \(provider)")
    let assertionRequest = provider.createCredentialAssertionRequest(
        challenge: challenge
    )
    print("performASPasskeyAuthorizationRequest assertionRequest: \(assertionRequest)")
    let assertion = try await controller.performRequest(assertionRequest)
    print("performASPasskeyAuthorizationRequest assertion: \(assertion)")
    guard case .passkeyAssertion(let passkeyAssertion) = assertion else {
        print("performASPasskeyAuthorizationRequest not a passkeyAssertion")
        fatalError("Unexpected authorization result: \(assertion)")
    }
    print("performASPasskeyAuthorizationRequest passkeyAssertion: \(passkeyAssertion)")
    return passkeyAssertion
}
