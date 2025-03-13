import AuthenticationServices
import CryptoKit
import Foundation
import ZKsyncSSO
import ZKsyncSSOFFI

public final class PasskeyAuthSync: PasskeyAuthenticator & Sendable {
    
    var manager: PasskeyManager {
        authenticator.manager
    }
    
    private let authenticator: PasskeyAsyncHelperAPI
    
    init(authenticator: PasskeyAsyncHelperAPI) {
        self.authenticator = authenticator
    }
    
    func signMessageAsync(message: Data) async throws -> Data {
        try await authenticator.authenticate(message: message)
    }
    
    public func signMessage(message: Data) throws -> Data {
        let semaphore = DispatchSemaphore(value: 0)
        var result: Result<Data, Error> = .failure(PasskeyError.unknown)
        
        Task {
            result = await Result {
                try await signMessageAsync(message: message)
            }
            semaphore.signal()
        }
        
        semaphore.wait()
        
        switch result {
        case .success(let data):
            return data
        case .failure(let error):
            throw error
        }
    }
}
