// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "ExamplePackage",
    platforms: [
        .iOS(.v18),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "ExamplePackage",
            targets: ["ExamplePackage"]),
    ],
    dependencies: [
       .package(path: "../../../ZKsyncSSO/"),
       .package(url: "https://github.com/pointfreeco/swift-sharing", from: "2.0.0")
    ],
    targets: [
        .target(
            name: "ExamplePackage",
            dependencies: [
              "ZKsyncSSO",
              "ExamplePackageUIComponents",
              .product(name: "Sharing", package: "swift-sharing"),
            ]),
        .target(
            name: "ExamplePackageUIComponents",
            dependencies: []),
        .testTarget(
            name: "ExamplePackageTests",
            dependencies: ["ExamplePackage"]
        ),
    ]
)
