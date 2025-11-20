# zksync-sso-auth-server

ZKsync SSO Auth Server

## How to deploy locally?

```sh
# Ensure era_test_node is already running (npx zksync-cli dev start)
# Deploy ZKsync SSO smart contracts
pnpm --dir packages/contracts run deploy

# Start Auth Server
pnpm nx dev auth-server
```

## Testing Guardian Recovery

See [GUARDIAN_TESTING.md](./GUARDIAN_TESTING.md) for a comprehensive manual
testing guide for the guardian recovery feature, including:

- Setting up SSO accounts as guardians
- Using external wallets as guardians
- Managing multiple guardians
- Edge cases and troubleshooting

## How to deploy to a new chain

If you are a ZKsync chain operator, there are a few more updates to make to
deploy.

Deploy the timestamp asserter and update the TimestampLocator.sol with your
chain id.

Adding contracts to the storage slot allow list (api_web3_json_rpc:
whitelisted_tokens_for_aa)

Include the addresses of SSO WebAuthValidator, SSO SessionKeyValidator, SSO
Beacon which need to be updated by the chain id in contractsByChain in the
client file. The block explorer url in the same file can also be updated if your
chain is listed in viem/chains.

## Design

The auth server is designed to run as a static front-end that facilitates
passkey signing and session creation via a trusted domain. The SDK has a wagmi
connector that points users to this project.

### Communication

The auth-server expects to managed via pop-up, so the client-auth-server uses
the pop-up communicator component to send the transaction request via
window.postMessage. The auth-server listens to this message and the loads the
UI, which then drives the user either through the sign-in with passkey or create
account flow.

It uses a custom RPC format to send success and status messages back to the SDK
for the session request data.

### Account Deployment

The account deployment and on-chain interaction for passkey signing and session
creation both happen within the auth server (session key signing happens
outside). This requires that the auth-server is connected to a paymaster, as the
deployment is performed from a random address for each deployment.

The auth server _currently_ uses the passkey credential id as the unique account
id if it's not provided, it then checks the account deployment factory to log a
user in as the factory stores a mapping from passkey id to account address.

### Dashboard

The auth server expects both the session key and passkey module to be installed,
so builds in dashboards for the modules to view basic information on sessions
and passkeys. It depends on the SDK's definition of SessionConfig to parse the
session for the user before approval. It reads the session status directly from
the chain and then relies on the SDK's contract ABI for the session key
validation module.

### Storage

To reduce user friction after signing in or signing up, the account information
is kept in browser local storage tied to the auth server domain.

### Wallet Connect

Wallet Connect has been integrated to facilitate connections with dApps that do
not utilize the ZKsync SSO SDK. This implementation currently exists as a Proof
of Concept (PoC) and should be regarded as experimental. As a PoC, it is subject
to ongoing modifications and may exhibit instability.
