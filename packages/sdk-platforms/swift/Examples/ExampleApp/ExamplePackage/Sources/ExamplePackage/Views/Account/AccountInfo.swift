import Foundation

protocol AccountInfoProviding: Codable {
    var name: String { get }
    var userID: String { get }
}

struct AccountInfo: Codable, Hashable, AccountInfoProviding {
    var name: String
    var userID: String
    var domain: String
}

struct DeployedAccount: Identifiable, Codable, Hashable, AccountInfoProviding {
    let info: AccountInfo
    let address: String
    let uniqueAccountId: String

    var name: String { info.name }
    var userID: String { info.userID }

    var id: String { address }
}
