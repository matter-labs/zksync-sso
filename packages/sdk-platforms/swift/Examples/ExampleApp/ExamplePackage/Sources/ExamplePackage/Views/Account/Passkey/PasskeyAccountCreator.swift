import Foundation
import AuthenticationServices
import ZKsyncSSO

struct PasskeyAccountCreator {
    private let manager: PasskeyManager
    
    init(manager: PasskeyManager) {
        self.manager = manager
    }
    
    func createAccount(
        userName: String,
        userID: String,
        challenge: Data
    ) async throws -> ZKsyncSSO.Account {
        let authorization = try await manager.createPasskey(
            userName: userName,
            userID: userID,
            challenge: challenge
        )
        
        let registrationData = try extractRegistrationData(from: authorization)
        
        let credentialId = registrationData.credentialID
        let rawClientDataJSON = registrationData.rawClientDataJSON
        
        let rpId = manager.relyingPartyIdentifier
        assert(rpId == "soo-sdk-example-pages.pages.dev")
        
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
                rpId: rpId,
                uniqueAccountId: userID
            )
        )
    }
    
    private func extractRegistrationData(from authorization: ASAuthorization) throws
        -> AuthorizationPlatformPublicKeyCredentialRegistration
    {
        let credentialType = AuthorizationCredentialType.from(
            credential: authorization.credential
        )
        
        guard credentialType.isPasskeyRegistration else {
            throw PasskeyError.invalidRegistrationData
        }
        
        switch credentialType {
        case .platformPublicKeyRegistration(let registration):
            return AuthorizationPlatformPublicKeyCredentialRegistration.from(
                registration: registration
            )
            
        case .passkeyRegistration:
            fatalError("Passkey registration not implemented")
            
        default:
            fatalError("Unexpected credential type")
        }
    }
} 
