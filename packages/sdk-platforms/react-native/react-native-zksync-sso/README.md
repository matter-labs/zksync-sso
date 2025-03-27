# react-native-zksync-sso

ZKsync Smart Sign On SDK for React Native

## Installation

```sh
npm install react-native-zksync-sso
```

## Usage

```ts
import {
    registerAccountWithUniqueId,
    generateRandomChallenge
} from 'react-native-zksync-sso';

const config = {
    contracts: {
        accountFactory: "0x...",
        passkey: "0x...",
        session: "0x...",
        accountPaymaster: "0x..."
    },
    nodeUrl: "https://..."
};

const accountInfo = {
    name: "Jane Doe",
    userID: "jdoe@example.com",
    domain: "example.com",
}

const challenge = generateRandomChallenge();

const secretAccountSalt = ...;

const deployedAccount = await registerAccountWithUniqueId(
    {
        name: accountInfo.name,
        userID: accountInfo.userID,
        rp: {
            name: accountInfo.domain,
            id: accountInfo.domain
        }
    },
    secretAccountSalt,
    challenge,
    config
);
```

## Contributing

See the [contributing guide](CONTRIBUTING.md) to learn how to contribute to the
repository and the development workflow.

## License

MIT

---

Made with
[create-react-native-library](https://github.com/callstack/react-native-builder-bob)
