// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "ZKsyncSSO",
    platforms: [
        .iOS(.v18),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "ZKsyncSSO",
            targets: ["ZKsyncSSO"])
    ],
    targets: [
        .target(
            name: "ZKsyncSSO",
            dependencies: ["ZKsyncSSOFFI"]),
        .target(
            name: "ZKsyncSSOFFI",
            dependencies: ["ffiFFI"]),
        .binaryTarget(
            name: "ffiFFI",
            path: "../../rust/zksync-sso/crates/ffi/out/ffiFFI.xcframework"),
        .testTarget(
            name: "ZKsyncSSOTests",
            dependencies: ["ZKsyncSSO"]),
    ]
)
