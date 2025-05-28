# react-native-zksync-sso

ZKsync Smart Sign On SDK for React Native

## Installation

```sh
npm install react-native-zksync-sso
```

## Usage

```ts
import { Platform } from 'react-native';
import sdk from 'react-native-zksync-sso';

const config = {
    contracts: {
        accountFactory: "0x...",
        passkey: "0x...",
        session: "0x...",
        accountPaymaster: "0x..."
    },
    nodeUrl: "https://...",
    deployWallet: {
        privateKeyHex: "..."
    }
};

const accountInfo = {
    name: "Jane Doe",
    userID: "jdoe@example.com",
};

const rpId = sdk.utils.createRpId(
    "example.com", // RP ID (same for both platforms)
    "android:apk-key-hash:your-app-key-hash" // Android origin
);

const challenge = sdk.utils.generateRandomChallenge();

const deployedAccount = await sdk.register.registerAccountWithUniqueId(
    {
        name: accountInfo.name,
        userID: accountInfo.userID,
        rp: {
            name: "example.com",
            id: rpId
        }
    },
    challenge,
    config
);
```

### Platform-Specific Configuration

#### iOS

For iOS, you can create an RP ID using:

```ts
const rpId = sdk.utils.createAppleRpId("example.com");
```

#### Android

For Android, you need both the RP ID and the app's origin (APK signature):

```ts
const rpId = sdk.utils.createAndroidRpId(
  "example.com",
  "android:apk-key-hash:your-app-signature-hash"
);
```

To get your Android app signature hash, see the [Android documentation on verifying
origin](https://developer.android.com/identity/sign-in/credential-manager#verify-origin).

### Available APIs

The SDK exports the following namespaced modules:

- `sdk.register` - Account registration functions
- `sdk.authenticate` - Authentication functions  
- `sdk.utils` - Utility functions including RpId creation

## ⚠️ Important: FFI Module Stability

The `sdk.ffi` module provides low-level bindings to the underlying Rust
implementation. **This interface is considered unstable and may change without
notice, even in minor version updates.**

**Do not use `sdk.ffi` directly in your application code.** Instead, use the
stable APIs provided by the `register`, `authenticate`, and `utils` modules,
which follow semantic versioning guarantees.

## Contributing

See the [contributing guide](CONTRIBUTING.md) to learn how to contribute to the
repository and the development workflow.

## License

MIT

---

Made with
[create-react-native-library](https://github.com/callstack/react-native-builder-bob)
