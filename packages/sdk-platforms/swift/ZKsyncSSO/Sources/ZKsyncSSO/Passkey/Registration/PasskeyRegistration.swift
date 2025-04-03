import Foundation
import AuthenticationServices
import SwiftUI

public func createAccount(
    userName: String,
    userID: String,
    secretAccountSalt: Data,
    challenge: Data,
    relyingPartyIdentifier: String,
    controller: AuthorizationController
) async throws -> Account {
    let authorization = try await performCredentialRegistrationRequest(
        userName: userName,
        userID: userID,
        challenge: challenge,
        relyingPartyIdentifier: relyingPartyIdentifier,
        controller: controller
    )
    
    let registrationData = try extractRegistrationData(from: authorization)
    
    let credentialId = registrationData.credentialID
    let rawClientDataJSON = registrationData.rawClientDataJSON
    
    if let jsonString = String(data: rawClientDataJSON, encoding: .utf8) {
        print("Raw Client Data JSON:")
        print(jsonString)
    }
    
    guard let credentialRawAttestationObject = registrationData.rawAttestationObject else {
        fatalError("Failed to extract raw attestation object from registration data")
    }
    
    print("Raw Attestation Object (hex):")
    print(credentialRawAttestationObject.map { String(format: "%02x", $0) }.joined())
    
    return try await ZKsyncSSO.deployAccountWith(
        params: .init(
            credentialRawAttestationObject: credentialRawAttestationObject,
            credentialRawClientDataJson: rawClientDataJSON,
            credentialId: credentialId,
            rpId: relyingPartyIdentifier,
            uniqueAccountId: userID
        ),
        secretAccountSalt: secretAccountSalt
    )
}

private func extractRegistrationData(
    from authorization: ASAuthorizationResult
) throws -> AuthorizationPlatformPublicKeyCredentialRegistration {
    guard case .passkeyRegistration(let registration) = authorization else {
        throw PasskeyError.invalidRegistrationData
    }
    return AuthorizationPlatformPublicKeyCredentialRegistration.from(
        registration: registration
    )
}

private func performCredentialRegistrationRequest(
    userName name: String,
    userID: String,
    challenge: Data,
    relyingPartyIdentifier: String,
    controller: AuthorizationController
) async throws -> ASAuthorizationResult {
    let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(
        relyingPartyIdentifier: relyingPartyIdentifier
    )
    let registrationRequest = provider.createCredentialRegistrationRequest(
        challenge: challenge,
        name: name,
        userID: Data(userID.utf8)
    )
    return try await controller.performRequest(registrationRequest)
}
