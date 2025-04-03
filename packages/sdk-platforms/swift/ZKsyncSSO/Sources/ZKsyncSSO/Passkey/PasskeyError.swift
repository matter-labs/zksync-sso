import Foundation

public enum PasskeyError: Error {
    case alreadyInitialized
    case invalidRegistrationData
    case unknown
    case invalidCredential
    case noPresentationAnchorAvailable
    case invalidSignature
}
