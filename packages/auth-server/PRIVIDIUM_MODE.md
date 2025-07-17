# Prividium Mode Implementation

This document describes the implementation of Prividium mode for the ZKsync SSO
auth server.

## Overview

Prividium mode is a security feature that requires users to authenticate with
Okta before accessing RPC endpoints. When enabled, all RPC requests will be
authenticated using a Bearer token obtained from Okta.

## Configuration

### Environment Variables

Set the following environment variables to enable Prividium mode:

```bash
# Enable Prividium mode
PRIVIDIUM_MODE=true

# Okta configuration
NUXT_PUBLIC_OKTA_ISSUER=https://your-okta-domain.okta.com/oauth2/default
NUXT_PUBLIC_OKTA_CLIENT_ID=your-okta-client-id
NUXT_PUBLIC_OKTA_REDIRECT_URI=http://localhost:3002/auth
NUXT_PUBLIC_PROXY_URL=https://your-authenticated-rpc-proxy.com
```

## Authentication Flow

When Prividium mode is enabled:

1. User visits the auth server
2. If not authenticated with Okta, they are redirected to the Okta login page
3. After successful authentication, user receives an ID token
4. The ID token is stored and used as a Bearer token for all RPC requests
5. User can then proceed with normal account creation/login flow
