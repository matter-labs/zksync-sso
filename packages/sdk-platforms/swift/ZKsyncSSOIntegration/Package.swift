// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "ZKsyncSSOIntegration",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(
            name: "ZKsyncSSOIntegration",
            targets: ["ZKsyncSSOIntegration"]
        ),
    ],
    dependencies: [
         .package(path: "../ZKsyncSSO/"),
    ],
    targets: [
        .target(
            name: "ZKsyncSSOIntegration",
            dependencies: ["ZKsyncSSO"]
        ),
        .testTarget(
            name: "ZKsyncSSOIntegrationTests",
            dependencies: ["ZKsyncSSOIntegration"]
        ),
    ]
)
