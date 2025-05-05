import Foundation
import AuthenticationServices
import SwiftUI
@preconcurrency import ZKsyncSSOFFI

public func handlePasskeyASAuthorization(
    result: ASAuthorizationResult
) throws -> ASAuthorizationPlatformPublicKeyCredentialAssertion {
    guard case .passkeyAssertion(let passkeyAssertion) = result else {
        throw PasskeyError.invalidCredential
    }
    return passkeyAssertion
}

public enum AuthorizationErrorWrapper: Error {
    case authorization(ASAuthorizationError)
    case unknown(Error)
}

public func performPasskeyAuthorizationRequest(
    relyingPartyIdentifier: String,
    userID: Data,
    challenge: Data,
    controller: AuthorizationController
) async -> Result<ASAuthorizationResult, AuthorizationErrorWrapper> {
    let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(
        relyingPartyIdentifier: relyingPartyIdentifier
    )
    let assertionRequest = provider.createCredentialAssertionRequest(
        challenge: challenge
    )
    do {
        let result: ASAuthorizationResult = try await controller.performRequest(
            assertionRequest
        )
        return .success(result)
    } catch let error as ASAuthorizationError {
        return .failure(.authorization(error))
    } catch {
        return .failure(.unknown(error))
    }
}

public func getAccountByUserId(
    uniqueAccountId: String,
    relyingPartyIdentifier: String,
) async throws -> Account {
    let account = try await ZKsyncSSOFFI.getAccountByUserId(
        uniqueAccountId: uniqueAccountId,
        config: Config.default.inner
    )
    return Account(
        address: account.address,
        uniqueAccountId: account.uniqueAccountId
    )
}
