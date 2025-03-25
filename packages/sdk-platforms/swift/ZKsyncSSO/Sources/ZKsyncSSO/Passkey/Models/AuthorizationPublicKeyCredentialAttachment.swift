import AuthenticationServices
import Foundation

public enum AuthorizationPublicKeyCredentialAttachment: String, Sendable, Codable {
    case platform
    case crossPlatform
    
    public static func from(
        attachment: ASAuthorizationPublicKeyCredentialAttachment
    ) -> Self {
        switch attachment {
        case .platform:
            return .platform
        case .crossPlatform:
            return .crossPlatform
        @unknown default:
            fatalError("Unknown attachment type: \(attachment)")
        }
    }
}
