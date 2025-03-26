import AuthenticationServices
import CryptoKit
import Foundation

public struct AuthorizationPublicKeyCredentialPRFRegistration {
    public let isSupported: Bool
    public let first: SymmetricKey?
    public let second: SymmetricKey?

    public static func from(registration: ASAuthorizationPublicKeyCredentialPRFRegistrationOutput)
        -> Self
    {
        Self(
            isSupported: registration.isSupported,
            first: registration.first,
            second: registration.second
        )
    }
}

public struct AuthorizationPublicKeyCredentialLargeBlobRegistration {
    public let isSupported: Bool

    public static func from(
        registration: ASAuthorizationPublicKeyCredentialLargeBlobRegistrationOutput
    ) -> Self {
        Self(isSupported: registration.isSupported)
    }
}

public struct AuthorizationPlatformPublicKeyCredentialRegistration {
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
        let prf = registration.prf
            .map(
                AuthorizationPublicKeyCredentialPRFRegistration.from(registration:)
            )

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

