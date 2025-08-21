// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "ExamplePackage",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(
            name: "ExamplePackage",
            targets: ["ExamplePackage"]
        ),
    ],
    dependencies: [
        .package(path: "../../../ZKsyncSSO/"),
        .package(path: "../../../ZKsyncSSOIntegration/"),
    ],
    targets: [
        .target(
            name: "ExamplePackage",
            dependencies: [
                "ZKsyncSSO",
                "ZKsyncSSOIntegration",
                "ExamplePackageUIComponents",
            ]
        ),
        .target(
            name: "ExamplePackageUIComponents",
            dependencies: []
        ),
        .testTarget(
            name: "ExamplePackageTests",
            dependencies: ["ExamplePackage"]
        ),
    ]
)
