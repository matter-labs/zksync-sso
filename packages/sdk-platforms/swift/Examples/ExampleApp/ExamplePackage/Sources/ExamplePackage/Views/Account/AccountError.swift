import Foundation

enum AccountError: Error {
    case alreadyDeployed
    case accountAlreadyCreated
    case noAccountAvailable
}
