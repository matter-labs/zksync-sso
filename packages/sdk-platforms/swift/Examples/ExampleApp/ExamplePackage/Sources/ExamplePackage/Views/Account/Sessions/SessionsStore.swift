import SwiftUI

@MainActor
class SessionsStore: ObservableObject {

    static let shared = SessionsStore()

    @Published private var sessionsByAccount: [String: [Session]] = [:]

    private init() {}
    
    static func preview() -> SessionsStore {
        return SessionsStore()
    }

    func getSessions(for accountAddress: String) -> [Session] {
        return sessionsByAccount[accountAddress] ?? []
    }

    func addSession(_ session: Session, for accountAddress: String) {
        if sessionsByAccount[accountAddress] == nil {
            sessionsByAccount[accountAddress] = []
        }
        sessionsByAccount[accountAddress]?.append(session)
    }

    func removeSession(withId sessionId: String, for accountAddress: String) {
        sessionsByAccount[accountAddress]?.removeAll { $0.id == sessionId }
    }
}
