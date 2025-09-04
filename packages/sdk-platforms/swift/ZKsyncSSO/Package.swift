// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "ZKsyncSSO",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(
            name: "ZKsyncSSO",
            targets: ["ZKsyncSSO"]
        ),
    ],
    targets: [
        .target(
            name: "ZKsyncSSO",
            dependencies: ["ZKsyncSSOFFI"],
            resources: [
                .copy("Config/config.json"),
            ]
        ),
        .target(
            name: "ZKsyncSSOFFI",
            dependencies: ["ZKsyncSSOCore"]
        ),
        .binaryTarget(
            name: "ZKsyncSSOCore",
            path: "../../rust/zksync-sso/crates/ffi/out/ZKsyncSSOCore.xcframework"
        ),
        .testTarget(
            name: "ZKsyncSSOTests",
            dependencies: ["ZKsyncSSO"]
        ),
    ]
)
