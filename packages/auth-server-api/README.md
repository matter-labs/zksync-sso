# Auth Server API

API responsible for deploying SSO smart accounts.

## Endpoints

- `POST /api/deploy-account` - Deploy a new smart account with passkey
  authentication
- `GET /api/health` - Health check endpoint

## Prividium Mode Setup

When Prividium mode is enabled, the auth server requires user authentication via
Prividium and routes all deployments through the Prividium RPC proxy.

### Configuration

Set the following environment variables to enable Prividium mode:

```env
PRIVIDIUM_MODE=true
PRIVIDIUM_RPC_PROXY_BASE_URL=https://rpc.prividium.io
PRIVIDIUM_PERMISSIONS_BASE_URL=https://permissions.prividium.io
PRIVIDIUM_ADMIN_PRIVATE_KEY=0x...  # Private key of a user with 'admin' role in Prividium
PRIVIDIUM_TEMPLATE_KEY=sso-smart-account  # Template key for whitelisting deployed contracts
SSO_AUTH_SERVER_BASE_URL=https://sso.example.com  # Base URL of the SSO auth server frontend (used as SIWE domain for admin authorization)
```

### Prividium Admin Panel Setup

1. Create a **Contract Permission Template** in the Prividium admin panel with
   the key `sso-smart-account` (or the value you set for
   `PRIVIDIUM_TEMPLATE_KEY`)
2. Configure the template with the required permissions listed below

### Contract ABI

The SSO smart account contract ABI is available at:
[`packages/erc4337-contracts/out/ModularSmartAccount.sol/ModularSmartAccount.json`](../erc4337-contracts/out/ModularSmartAccount.sol/ModularSmartAccount.json)
after compiling the contracts.

### Required Permissions

Configure the following method permissions in your contract template:

| Function          | Permission Level | Access Details |
| ----------------- | ---------------- | -------------- |
| `ENTRY_POINT_V08` | All Users        |                |
| `accountId`       | All Users        |                |
| `domainSeparator` | All Users        |                |
| `eip712Domain`    | All Users        |                |
| `entryPoint`      | All Users        |                |
