import SwiftUI
import ZKsyncSSO

struct Session: Identifiable, Hashable {
    var id: String {
        return sessionHash
    }

    let createdAt: Date
    let sessionHash: String
    let sessionSpec: SessionSpec
    let sessionKey: String

    init(
        createdAt: Date,
        sessionSpec: SessionSpec,
        sessionKey: String
    ) {
        self.createdAt = createdAt
        self.sessionHash = try! sessionSpec.sessionHash()
        self.sessionSpec = sessionSpec
        self.sessionKey = sessionKey
    }
    
    static func create(
        sessionKey: String,
        sessionSpec: SessionSpec,
    ) -> Self {
        Self(
            createdAt: Date(),
            sessionSpec: sessionSpec,
            sessionKey: sessionKey
        )
    }
}

extension SessionSpec {
    static var `default`: SessionSpec {
        return SessionSpec(
            signer: "0x9BbC92a33F193174bf6Cc09c4b4055500d972479",
            expiresAt: String(Int(Date().addingTimeInterval(86400).timeIntervalSince1970)),  // 24 hours
            feeLimit: UsageLimit(
                limitType: .lifetime,
                limit: "100000000000000000",
                period: "0"
            ),
            callPolicies: [],
            transferPolicies: [
                TransferSpec(
                    target: "0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72",
                    maxValuePerUse: "10000000000000000",
                    valueLimit: UsageLimit(
                        limitType: .unlimited,
                        limit: "0",
                        period: "0"
                    )
                )
            ]
        )
    }

    static func initialSession(sessionOwner: EOASigner) -> SessionSpec {
        return SessionSpec(
            signer: sessionOwner.address,
            expiresAt: String(Int(Date().addingTimeInterval(86400).timeIntervalSince1970)),  // 24 hours
            feeLimit: UsageLimit(
                limitType: .lifetime,
                limit: "100000000000000000",
                period: "0"
            ),
            callPolicies: [],
            transferPolicies: [
                TransferSpec(
                    target: "0xdeBbD4CE2Bd6BD869D3ac93666A0D5F4fc06FC72",
                    maxValuePerUse: "10000000000000000",
                    valueLimit: UsageLimit(
                        limitType: .unlimited,
                        limit: "0",
                        period: "0"
                    )
                )
            ]
        )
    }
    
    func with(expiry: Date) -> SessionSpec {
        var copy = self
        copy.expiresAt = String(Int(expiry.timeIntervalSince1970))
        return copy
    }
}
