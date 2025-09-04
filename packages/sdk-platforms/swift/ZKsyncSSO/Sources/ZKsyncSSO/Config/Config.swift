import Foundation
import ZKsyncSSOFFI

public struct Config {
    
    public var contracts: SsoContracts { SsoContracts(inner: inner.contracts) }
    
    public var nodeUrl: String { inner.nodeUrl }
    
    public var deployWallet: DeployWallet? {
        get { inner.deployWallet.map(DeployWallet.init) }
        set { inner.deployWallet = newValue.map(\.inner) }
    }

    var inner: ZKsyncSSOFFI.Config

    public init(
        contracts: SsoContracts,
        nodeUrl: String,
        deployWallet: DeployWallet?
    ) {
        inner = .init(
            contracts: contracts.inner,
            nodeUrl: nodeUrl,
            deployWallet: deployWallet.map(\.inner)
        )
    }

    public static var `default`: Self {
        let innerDefault = ZKsyncSSOFFI.Config.default
        return Self(
            contracts: SsoContracts(inner: innerDefault.contracts),
            nodeUrl: innerDefault.nodeUrl,
            deployWallet: {
                guard let privateKeyHex = innerDefault.deployWallet?.privateKeyHex else {
                    return nil
                }
                return DeployWallet(privateKeyHex: privateKeyHex)
            }()
        )
    }
}

extension ZKsyncSSOFFI.Config {
    static var `default`: Self {
        guard let configUrl = Bundle.module.url(forResource: "config", withExtension: "json") else {
            fatalError("config url couldn't be read")
        }

        guard let data = try? Data(contentsOf: configUrl) else {
            fatalError("\(configUrl) data couldn't be read")
        }

        let decoder = JSONDecoder()

        guard let config = try? decoder.decode(Self.self, from: data) else {
            fatalError("config.json data coulnd't be read")
        }

        return config
    }
}
