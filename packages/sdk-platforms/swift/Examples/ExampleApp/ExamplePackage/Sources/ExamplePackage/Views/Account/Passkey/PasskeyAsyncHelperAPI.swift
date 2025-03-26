import Foundation
import AuthenticationServices
import ZKsyncSSO

protocol PasskeyAsyncHelperAPI: Sendable {
    var manager: PasskeyManager { get }
    
    func authenticate(message: Data) async throws -> Data
}
