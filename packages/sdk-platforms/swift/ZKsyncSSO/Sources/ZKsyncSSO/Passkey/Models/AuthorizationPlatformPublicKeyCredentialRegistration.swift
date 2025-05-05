import AuthenticationServices
import CryptoKit
import Foundation

public struct AuthorizationPublicKeyCredentialPRFRegistration: Sendable {
    public let isSupported: Bool
    public let first: SendableSymmetricKey?
    public let second: SendableSymmetricKey?

    @available(iOS 18.0, *)
    public static func from(
        registration: ASAuthorizationPublicKeyCredentialPRFRegistrationOutput
    ) -> Self {
        Self(
            isSupported: registration.isSupported,
            first: registration.first.map(SendableSymmetricKey.from),
            second: registration.second.map(SendableSymmetricKey.from)
        )
    }
}

public struct AuthorizationPublicKeyCredentialLargeBlobRegistration: Sendable {
    public let isSupported: Bool

    public static func from(
        registration: ASAuthorizationPublicKeyCredentialLargeBlobRegistrationOutput
    ) -> Self {
        Self(isSupported: registration.isSupported)
    }
}

public struct AuthorizationPlatformPublicKeyCredentialRegistration: Sendable {
    
    public let attachment: AuthorizationPublicKeyCredentialAttachment

    public let credentialID: Data

    public let largeBlob: AuthorizationPublicKeyCredentialLargeBlobRegistration?

    public let prf: AuthorizationPublicKeyCredentialPRFRegistration?

    public let rawAttestationObject: Data?

    public let rawClientDataJSON: Data

    public static func from(
        registration: ASAuthorizationPlatformPublicKeyCredentialRegistration
    ) -> Self {
        let largeBlob = registration.largeBlob
            .map(
                AuthorizationPublicKeyCredentialLargeBlobRegistration.from(
                    registration:
                )
            )
        var prf: AuthorizationPublicKeyCredentialPRFRegistration? = nil
        if #available(iOS 18.0, *) {
            prf = registration.prf
                .map(
                    AuthorizationPublicKeyCredentialPRFRegistration
                        .from(registration:)
                )
        }

        return AuthorizationPlatformPublicKeyCredentialRegistration(
            attachment: AuthorizationPublicKeyCredentialAttachment.from(
                attachment: registration.attachment
            ),
            credentialID: registration.credentialID,
            largeBlob: largeBlob,
            prf: prf,
            rawAttestationObject: registration.rawAttestationObject,
            rawClientDataJSON: registration.rawClientDataJSON
        )
    }
}
