import AuthenticationServices
import Foundation

public enum AuthorizationCredentialType {
    case appleID(ASAuthorizationAppleIDCredential)
    case platformPublicKeyAssertion(ASAuthorizationPlatformPublicKeyCredentialAssertion)
    case platformPublicKeyRegistration(ASAuthorizationPlatformPublicKeyCredentialRegistration)
    case securityKeyPublicKeyAssertion(ASAuthorizationSecurityKeyPublicKeyCredentialAssertion)
    case securityKeyPublicKeyRegistration(ASAuthorizationSecurityKeyPublicKeyCredentialRegistration)
    case singleSignOn(ASAuthorizationSingleSignOnCredential)
    case passkeyAssertion(ASPasskeyAssertionCredential)
    case passkeyRegistration(ASPasskeyRegistrationCredential)
    case password(ASPasswordCredential)

    public static func from(credential: ASAuthorizationCredential) -> AuthorizationCredentialType {
        switch credential {
        case let credential as ASAuthorizationAppleIDCredential:
            return .appleID(credential)
        case let credential as ASAuthorizationPlatformPublicKeyCredentialAssertion:
            return .platformPublicKeyAssertion(credential)
        case let credential as ASAuthorizationPlatformPublicKeyCredentialRegistration:
            return .platformPublicKeyRegistration(credential)
        case let credential as ASAuthorizationSecurityKeyPublicKeyCredentialAssertion:
            return .securityKeyPublicKeyAssertion(credential)
        case let credential as ASAuthorizationSecurityKeyPublicKeyCredentialRegistration:
            return .securityKeyPublicKeyRegistration(credential)
        case let credential as ASAuthorizationSingleSignOnCredential:
            return .singleSignOn(credential)
        case let credential as ASPasskeyAssertionCredential:
            return .passkeyAssertion(credential)
        case let credential as ASPasskeyRegistrationCredential:
            return .passkeyRegistration(credential)
        case let credential as ASPasswordCredential:
            return .password(credential)
        default:
            fatalError("Unexpected credential type: \(type(of: credential))")
        }
    }
}

extension AuthorizationCredentialType {
    public var credential: ASAuthorizationCredential {
        switch self {
        case .appleID(let credential): return credential
        case .platformPublicKeyAssertion(let credential): return credential
        case .platformPublicKeyRegistration(let credential): return credential
        case .securityKeyPublicKeyAssertion(let credential): return credential
        case .securityKeyPublicKeyRegistration(let credential): return credential
        case .singleSignOn(let credential): return credential
        case .passkeyAssertion(let credential): return credential
        case .passkeyRegistration(let credential): return credential
        case .password(let credential): return credential
        }
    }

    public var isPasskeyRegistration: Bool {
        switch self {
        case .platformPublicKeyRegistration, .passkeyRegistration:
            return true
        default:
            return false
        }
    }

    public var isPasskeyAssertion: Bool {
        switch self {
        case .platformPublicKeyAssertion, .passkeyAssertion:
            return true
        default:
            return false
        }
    }
}
