#!/bin/sh

set -eo pipefail

pushd `dirname $0`
trap popd EXIT

CRATE_NAME="ffi"
VERSION=${1:-"1.0"} # first arg or "1.0"
REVERSE_DOMAIN="io.matterlabs"
BUNDLE_IDENTIFIER="$REVERSE_DOMAIN.$FRAMEWORK_LIBRARY_NAME"
LIBRARY_NAME="lib$CRATE_NAME.a"
FRAMEWORK_LIBRARY_NAME="ZKsyncSSOCore"
FRAMEWORK_NAME="$FRAMEWORK_LIBRARY_NAME.framework"
XC_FRAMEWORK_NAME="$FRAMEWORK_LIBRARY_NAME.xcframework"
HEADER_NAME="$FRAMEWORK_LIBRARY_NAME.h"
# Generated file names from uniffi
GENERATED_HEADER="${CRATE_NAME}FFI.h"
GENERATED_SWIFT="$CRATE_NAME.swift"
OUT_PATH="out"
MIN_IOS_VERSION="17.0"
WRAPPER_PATH="../../../../swift/ZKsyncSSO/Sources/ZKsyncSSOFFI"
TARGET_PATH="../../target"
BUILD_TYPE="debug" # use debug during development

AARCH64_APPLE_IOS_SIM_PATH="$TARGET_PATH/aarch64-apple-ios-sim/$BUILD_TYPE"

# Build only for aarch64-apple-ios-sim (Simulator on Apple Silicon)
echo "Building for aarch64-apple-ios-sim..."
rustup target add aarch64-apple-ios-sim
cargo build --target aarch64-apple-ios-sim

# Generate swift wrapper
echo "Generating swift wrapper..."
mkdir -p $OUT_PATH
mkdir -p $WRAPPER_PATH
cargo run --features=uniffi/cli --bin uniffi-bindgen generate \
    --library $AARCH64_APPLE_IOS_SIM_PATH/$LIBRARY_NAME \
    --language swift \
    --out-dir $OUT_PATH

# Rename the generated header file to match our framework name
mv $OUT_PATH/$GENERATED_HEADER $OUT_PATH/$HEADER_NAME

# Create framework for simulator
echo "Creating framework for simulator..."
rm -rf $OUT_PATH/$FRAMEWORK_NAME
mkdir -p $OUT_PATH/$FRAMEWORK_NAME/Headers
mkdir -p $OUT_PATH/$FRAMEWORK_NAME/Modules
cp $OUT_PATH/$HEADER_NAME $OUT_PATH/$FRAMEWORK_NAME/Headers
cp $AARCH64_APPLE_IOS_SIM_PATH/$LIBRARY_NAME $OUT_PATH/$FRAMEWORK_NAME/$FRAMEWORK_LIBRARY_NAME
cat <<EOT > $OUT_PATH/$FRAMEWORK_NAME/Modules/module.modulemap
framework module $FRAMEWORK_LIBRARY_NAME {
  umbrella header "$HEADER_NAME"

  export *
  module * { export * }
}
EOT

cat <<EOT > $OUT_PATH/$FRAMEWORK_NAME/Info.plist
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleExecutable</key>
	<string>$FRAMEWORK_LIBRARY_NAME</string>
	<key>CFBundleIdentifier</key>
	<string>$BUNDLE_IDENTIFIER</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>$FRAMEWORK_LIBRARY_NAME</string>
	<key>CFBundlePackageType</key>
	<string>FMWK</string>
	<key>CFBundleShortVersionString</key>
	<string>1.0</string>
	<key>CFBundleVersion</key>
	<string>$VERSION</string>
	<key>NSPrincipalClass</key>
	<string></string>
	<key>MinimumOSVersion</key>
	<string>$MIN_IOS_VERSION</string>
</dict>
</plist>
EOT

# Create xcframework
echo "Creating xcframework..."
rm -rf $OUT_PATH/$XC_FRAMEWORK_NAME
xcodebuild -create-xcframework \
    -framework $OUT_PATH/$FRAMEWORK_NAME \
    -output $OUT_PATH/$XC_FRAMEWORK_NAME

# Copy and adjust swift wrapper
echo "Copying and adjusting swift wrapper..."
cat <<EOT > $OUT_PATH/import.txt
#if os(macOS)
import SystemConfiguration
#endif
EOT
mv $OUT_PATH/$GENERATED_SWIFT $OUT_PATH/$FRAMEWORK_LIBRARY_NAME.swift
sed -i '' "s/canImport(${CRATE_NAME}FFI)/canImport($FRAMEWORK_LIBRARY_NAME)/" "$OUT_PATH/$FRAMEWORK_LIBRARY_NAME.swift"
sed -i '' "s/import ${CRATE_NAME}FFI/import $FRAMEWORK_LIBRARY_NAME/" "$OUT_PATH/$FRAMEWORK_LIBRARY_NAME.swift"
cat $OUT_PATH/import.txt $OUT_PATH/$FRAMEWORK_LIBRARY_NAME.swift > $WRAPPER_PATH/$FRAMEWORK_LIBRARY_NAME.swift

# Apply necessary fixes to the swift wrapper
sed -i '' 's/private var initializationResult: InitializationResult = {/private let initializationResult: InitializationResult = {/' "$WRAPPER_PATH/$FRAMEWORK_LIBRARY_NAME.swift"
sed -i '' 's/private let uniffiContinuationHandleMap = UniffiHandleMap<UnsafeContinuation<Int8, Never>>()/nonisolated(unsafe) private let uniffiContinuationHandleMap = UniffiHandleMap<UnsafeContinuation<Int8, Never>>()/' "$WRAPPER_PATH/$FRAMEWORK_LIBRARY_NAME.swift"
echo "// swift-format-ignore-file" | cat - "$WRAPPER_PATH/$FRAMEWORK_LIBRARY_NAME.swift" > temp && mv temp "$WRAPPER_PATH/$FRAMEWORK_LIBRARY_NAME.swift"
sed -i '' 's/open class Client:/open class Client:\
    @unchecked Sendable,/' "$WRAPPER_PATH/$FRAMEWORK_LIBRARY_NAME.swift"
sed -i '' 's/private class UniffiHandleMap<T> {/private class UniffiHandleMap<T>: @unchecked Sendable {/' "$WRAPPER_PATH/$FRAMEWORK_LIBRARY_NAME.swift"
sed -i '' 's/static var vtable: UniffiVTableCallbackInterfacePasskeyAuthenticator = .init(/nonisolated(unsafe) static var vtable: UniffiVTableCallbackInterfacePasskeyAuthenticator = .init(/' "$WRAPPER_PATH/$FRAMEWORK_LIBRARY_NAME.swift"
sed -i '' 's/fileprivate static var handleMap = UniffiHandleMap<PasskeyAuthenticator>()/fileprivate static let handleMap = UniffiHandleMap<PasskeyAuthenticator>()/' "$WRAPPER_PATH/$FRAMEWORK_LIBRARY_NAME.swift"
sed -i '' 's/filenonisolated(unsafe) private let uniffiContinuationHandleMap = UniffiHandleMap<UnsafeContinuation<Int8, Never>>()/nonisolated(unsafe) fileprivate let uniffiContinuationHandleMap = UniffiHandleMap<UnsafeContinuation<Int8, Never>>()/' "$WRAPPER_PATH/$FRAMEWORK_LIBRARY_NAME.swift"
cat <<EOT >> "$WRAPPER_PATH/$FRAMEWORK_LIBRARY_NAME.swift"

extension Config: Codable {}
extension PasskeyContracts: Codable {}
extension DeployWallet: Codable {}
EOT