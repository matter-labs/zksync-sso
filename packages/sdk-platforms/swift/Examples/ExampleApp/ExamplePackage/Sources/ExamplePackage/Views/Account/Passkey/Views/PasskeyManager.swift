import AuthenticationServices
import Combine
import Foundation
import Observation
import SwiftUI

#if os(iOS)
    import UIKit
#endif
#if os(macOS)
    import AppKit
#endif

enum PasskeyError: Error {
    case alreadyInitialized
    case invalidRegistrationData
    case unknown
    case invalidCredential
    case noPresentationAnchorAvailable
    case invalidSignature
}

@MainActor
@Observable
public final class PasskeyManager: NSObject {
    public let relyingPartyIdentifier: String
    private var pendingRequests: [UUID: PendingRequest] = [:]
    private var presentationContextProvider: PresentationContextProvider?

    private struct PendingRequest {
        let controller: ASAuthorizationController
        let completion: (Result<ASAuthorization, Error>) -> Void
    }

    public init(relyingPartyIdentifier: String) {
        self.relyingPartyIdentifier = relyingPartyIdentifier
        super.init()
    }

    public func createPasskey(
        userName: String,
        userID: String,
        challenge: Data
    ) async throws -> ASAuthorization {
        guard let presentationContextProvider else {
            throw PasskeyError.noPresentationAnchorAvailable
        }

        return try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                assert(relyingPartyIdentifier == "soo-sdk-example-pages.pages.dev")
                
                let provider = ASAuthorizationPlatformPublicKeyCredentialProvider(
                    relyingPartyIdentifier: relyingPartyIdentifier
                )
                let request = provider.createCredentialRegistrationRequest(
                    challenge: challenge,
                    name: userName,
                    userID: Data(userID.utf8)
                )

                let controller = ASAuthorizationController(authorizationRequests: [request])
                controller.delegate = self
                controller.presentationContextProvider = presentationContextProvider

                let requestId = UUID()
                pendingRequests[requestId] = PendingRequest(
                    controller: controller,
                    completion: { result in
                        self.pendingRequests.removeValue(forKey: requestId)
                        continuation.resume(with: result)
                    }
                )

                controller.performRequests()
            }
        } onCancel: { [self] in
            Task { @MainActor in
                for request in pendingRequests.values {
                    request.controller.cancel()
                }
                pendingRequests.removeAll()
            }
        }
    }

    public func authenticate(
        message: Data
    ) async throws -> ASAuthorizationPlatformPublicKeyCredentialAssertion {
        try await withTaskCancellationHandler {
            assert(relyingPartyIdentifier == "soo-sdk-example-pages.pages.dev")
            
            let assertionRequest = ASAuthorizationPlatformPublicKeyCredentialProvider(
                relyingPartyIdentifier: relyingPartyIdentifier
            )
            .createCredentialAssertionRequest(challenge: message)

            return try await performAuthorizationRequest(assertionRequest)
        } onCancel: { [self] in
            Task { @MainActor in
                for request in pendingRequests.values {
                    request.controller.cancel()
                }
                pendingRequests.removeAll()
            }
        }
    }

    private func performAuthorizationRequest<T: ASAuthorizationCredential>(
        _ request: ASAuthorizationRequest
    ) async throws -> T {
        print(
            "Performing auth request, context provider: \(String(describing: presentationContextProvider))"
        )
        guard let contextProvider = presentationContextProvider else {
            print("❌ No presentation context provider available")
            throw PasskeyError.noPresentationAnchorAvailable
        }

        return try await withCheckedThrowingContinuation { continuation in
            let controller = ASAuthorizationController(authorizationRequests: [request])
            controller.delegate = self
            controller.presentationContextProvider = contextProvider

            let requestId = UUID()

            pendingRequests[requestId] = PendingRequest(
                controller: controller,
                completion: { result in
                    self.pendingRequests.removeValue(forKey: requestId)
                    if case .success(let authorization) = result,
                        let credential = authorization.credential as? T
                    {
                        continuation.resume(returning: credential)
                    } else if case .failure(let error) = result {
                        continuation.resume(throwing: error)
                    } else {
                        continuation.resume(throwing: PasskeyError.invalidCredential)
                    }
                }
            )

            controller.performRequests()
        }
    }

    func setPresentationAnchor(_ window: ASPresentationAnchor?) {
        print("Setting presentation anchor: \(String(describing: window))")
        if let window {
            self.presentationContextProvider = PresentationContextProvider(anchor: window)
        } else {
            self.presentationContextProvider = nil
        }
    }
}

extension PasskeyManager: ASAuthorizationControllerDelegate {
    public func authorizationController(
        controller: ASAuthorizationController,
        didCompleteWithAuthorization authorization: ASAuthorization
    ) {
        if let matchingRequest = pendingRequests.values.first(where: {
            $0.controller === controller
        }) {
            matchingRequest.completion(.success(authorization))
        }
    }

    public func authorizationController(
        controller: ASAuthorizationController,
        didCompleteWithError error: Error
    ) {
        if let matchingRequest = pendingRequests.values.first(where: {
            $0.controller === controller
        }) {
            matchingRequest.completion(.failure(error))
        }
    }
}

private class PresentationContextProvider: NSObject,
    ASAuthorizationControllerPresentationContextProviding
{
    let anchor: ASPresentationAnchor

    init(anchor: ASPresentationAnchor) {
        self.anchor = anchor
        super.init()
    }

    func presentationAnchor(for controller: ASAuthorizationController) -> ASPresentationAnchor {
        anchor
    }
}
