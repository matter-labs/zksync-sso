import Foundation
import Observation
import Sharing

extension URL {
    static var accounts: URL {
        Self.documentsDirectory.appending(component: "accounts.json")
    }
}

@Observable
class AccountStore {

    @ObservationIgnored
    @Shared(.fileStorage(.accounts)) private(set) var deployedAccounts: [DeployedAccount] = []

    init() {}

    public func accountDeployed(_ account: DeployedAccount) throws {
        $deployedAccounts.withLock { deployedAccounts in
            deployedAccounts.append(account)
        }
    }

    public static func generateUserID() -> String {
        UUID().uuidString.replacingOccurrences(of: "-", with: "")
    }
}

extension Array {

    subscript(safe index: Int) -> Element? {
        guard index >= 0 && index < count else { return nil }
        return self[index]
    }
}

protocol AccountInfoProviding: Codable {
    var name: String { get }
    var userID: String { get }
}

struct AccountInfo: Codable, Hashable, AccountInfoProviding {
    let name: String
    let userID: String
    let domain: String
}

struct DeployedAccount: Identifiable, Codable, Hashable, AccountInfoProviding {
    let info: AccountInfo
    let address: String
    let uniqueAccountId: String
    
    var name: String { info.name }
    var userID: String { info.userID }
    
    var id: String { address }
}

struct Account: Codable, Identifiable, AccountInfoProviding {
    
    var id: String {
        address ?? userID
    }

    enum State: Codable, AccountInfoProviding {
        
        case deployed(DeployedAccount)
        case inital(AccountInfo)

        var name: String {
            switch self {
            case .deployed(let deployed):
                return deployed.name
            case .inital(let info):
                return info.name
            }
        }

        var userID: String {
            switch self {
            case .deployed(let deployed):
                return deployed.userID
            case .inital(let info):
                return info.userID
            }
        }

        var address: String? {
            switch self {
            case .deployed(let deployed):
                return deployed.address
            case .inital:
                return nil
            }
        }
    }

    var name: String {
        state.name
    }

    var userID: String {
        state.userID
    }

    var address: String? {
        state.address
    }

    var state: State

    init(userID: String, name: String, domain: String) {
        self.state = .inital(.init(name: name, userID: userID, domain: domain))
    }

    mutating func deploy(to address: String, uniqueAccountId: String) throws {
        switch state {
        case .inital(let info):
            state = .deployed(
                .init(
                    info: info,
                    address: address,
                    uniqueAccountId: uniqueAccountId
                )
            )
        case .deployed:
            throw AccountError.alreadyDeployed
        }
    }
}

enum AccountError: Error {
    case alreadyDeployed
    case accountAlreadyCreated
    case noAccountAvailable
}

extension String {
    var formattedUserID: String {
        guard count > 12 else { return self }
        let prefix = prefix(6)
        let suffix = suffix(6)
        return "\(prefix)...\(suffix)"
    }

    var formattedAddress: String {
        guard count > 12 else { return self }
        let prefix = prefix(6)
        let suffix = suffix(4)
        return "\(prefix)...\(suffix)"
    }
}
