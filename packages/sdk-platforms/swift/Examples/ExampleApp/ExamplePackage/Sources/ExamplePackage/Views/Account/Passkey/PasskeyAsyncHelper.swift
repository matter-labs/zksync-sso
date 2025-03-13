import Foundation
import AuthenticationServices
import ZKsyncSSO

struct PasskeyAuthenticatorHelper: PasskeyAsyncHelperAPI {
    public let manager: PasskeyManager
    
    init(manager: PasskeyManager) {
        self.manager = manager
    }
    
    func authenticate(message: Data) async throws -> Data {
        let assertion = try await manager.authenticate(message: message)
        
        print("Got assertion: \(assertion)")
        
        let authAssertion = AuthorizationPlatformPublicKeyCredentialAssertion.from(
            assertion: assertion
        )
        
        let authAssertionData = try JSONEncoder().encode(authAssertion)
        
        print(String(data: authAssertionData, encoding: .utf8) ?? "Couldn't decode assertion data")
        
        return authAssertionData
    }
}
