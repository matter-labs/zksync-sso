import AuthenticationServices
import CryptoKit
import Foundation

public struct AuthorizationPublicKeyCredentialLargeBlobAssertionOutput: Codable {

    public enum OperationResult: Codable {
        case read(data: Data?)
        case write(success: Bool)

        public init(result: OperationResult) {
            self = result
        }

        public static func from(
            result: ASAuthorizationPublicKeyCredentialLargeBlobAssertionOutput.OperationResult
        ) -> Self {
            switch result {
            case .read(let data):
                return .init(result: .read(data: data))
            case .write(let success):
                return .init(result: .write(success: success))
            @unknown default:
                fatalError()
            }
        }
    }

    public let result: OperationResult

    public static func from(
        assertionOutput: ASAuthorizationPublicKeyCredentialLargeBlobAssertionOutput
    ) -> Self {
        Self(
            result: OperationResult.from(result: assertionOutput.result)
        )
    }

    public static func read(data: Data?) -> Self {
        from(assertionOutput: .read(data: data))
    }

    public static func write(success: Bool) -> Self {
        .from(assertionOutput: .write(success: success))
    }
}

public struct AuthorizationPublicKeyCredentialPRFAssertionOutput: Codable {

    public let first: SymmetricKey

    public let second: SymmetricKey?

    public init(first: SymmetricKey, second: SymmetricKey?) {
        self.first = first
        self.second = second
    }

    public static func from(
        assertionOutput: ASAuthorizationPublicKeyCredentialPRFAssertionOutput
    ) -> Self {
        Self(first: assertionOutput.first, second: assertionOutput.second)
    }
}

extension SymmetricKey: Codable {
    public init(from decoder: any Decoder) throws {
        let container = try decoder.singleValueContainer()
        let data = try container.decode(Data.self)
        self = SymmetricKey(data: data)
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        withUnsafeBytes { bytes in
            let data = Data(bytes)
            try? container.encode(data)
        }
    }
}

public struct AuthorizationPlatformPublicKeyCredentialAssertion: Codable {

    public let attachment: AuthorizationPublicKeyCredentialAttachment

    public let rawAuthenticatorData: Data

    public let userID: Data

    public let signature: Data

    public let credentialID: Data

    public let rawClientDataJSON: Data

    public let largeBlob: AuthorizationPublicKeyCredentialLargeBlobAssertionOutput?

    public let prf: AuthorizationPublicKeyCredentialPRFAssertionOutput?

    public static func from(
        assertion: ASAuthorizationPlatformPublicKeyCredentialAssertion
    ) -> Self {
        let attachment = AuthorizationPublicKeyCredentialAttachment.from(
            attachment: assertion.attachment
        )

        let largeBlob = assertion.largeBlob
            .map(
                AuthorizationPublicKeyCredentialLargeBlobAssertionOutput.from(
                    assertionOutput:
                )
            )

        let prf = assertion.prf
            .map(
                AuthorizationPublicKeyCredentialPRFAssertionOutput.from(
                    assertionOutput:
                )
            )

        return Self(
            attachment: attachment,
            rawAuthenticatorData: assertion.rawAuthenticatorData,
            userID: assertion.userID,
            signature: assertion.signature,
            credentialID: assertion.credentialID,
            rawClientDataJSON: assertion.rawClientDataJSON,
            largeBlob: largeBlob,
            prf: prf
        )
    }
}
