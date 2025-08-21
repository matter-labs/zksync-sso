import Foundation
import ZKsyncSSO

public struct RichWallet: Signer, Codable, Sendable {
    public let address: String
    public let privateKeyHex: String

    public init(address: String, privateKeyHex: String) {
        self.address = address
        self.privateKeyHex = privateKeyHex
    }
}

extension RichWallet {
    public static var zero: Self {
        Self(
            address: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
            privateKeyHex: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
        )
    }
    
    public static var one: Self {
        Self(
            address: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
            privateKeyHex: "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"
        )
    }
    
    public static var two: Self {
        Self(
            address: "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC",
            privateKeyHex: "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"
        )
    }
    
    public static var three: Self {
        Self(
            address: "0x90F79bf6EB2c4f870365E785982E1f101E93b906",
            privateKeyHex: "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6"
        )
    }
    
    public static var four: Self {
        Self(
            address: "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65",
            privateKeyHex: "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a"
        )
    }
    
    public static var five: Self {
        Self(
            address: "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc",
            privateKeyHex: "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba"
        )
    }
    
    public static var six: Self {
        Self(
            address: "0x976EA74026E726554dB657fA54763abd0C3a0aa9",
            privateKeyHex: "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e"
        )
    }
    
    public static var seven: Self {
        Self(
            address: "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955",
            privateKeyHex: "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356"
        )
    }
    
    public static var eight: Self {
        Self(
            address: "0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f",
            privateKeyHex: "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97"
        )
    }
    
    public static var nine: Self {
        Self(
            address: "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720",
            privateKeyHex: "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6"
        )
    }
}
