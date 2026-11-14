# OAuth2 & OIDC Identity Architecture & Setup Guide

This document describes the design, security invariants, provider configuration, and Nuxt BFF integration for Mensetsu's multi-provider OAuth2 and OpenID Connect (OIDC) identity subsystem.

---

## 1. Architectural Topology & Authentication Flow

Mensetsu uses a secure, BFF-centric OAuth2/OIDC handoff flow designed to prevent credential leakage in URLs, browser histories, or client-side storage, while supporting multi-provider login (Google, GitHub, Facebook, and custom OIDC providers).

### 1.1 Sequence Diagram

```mermaid
sequenceDiagram
    autonumber
    actor User as User / Browser
    participant BFF as Nuxt BFF (/api/auth/...)
    participant API as Rust API Core
    participant IdP as Identity Provider (Google, GitHub, etc.)
    participant DB as SurrealDB

    User->>BFF: Click sign-in with provider
    BFF->>API: GET /api/v1/auth/oauth/authorize/{provider_key}
    API->>DB: Store state, encrypted PKCE verifier, nonce, expiry
    API-->>User: 303 redirect to IdP; set short-lived state cookie

    User->>IdP: Authenticate and authorize
    IdP-->>User: Redirect to API callback with code and state
    User->>API: GET /api/v1/auth/oauth/callback?code=...&state=...
    API->>DB: Validate one-use state and load provider config
    API->>IdP: SSRF-validated token exchange and userinfo/email fetch
    API->>DB: Resolve linked identity; refuse implicit email linking
    API->>DB: Store one-use auth handoff code
    API-->>BFF: 303 redirect to configured web callback with code

    BFF->>API: POST /api/v1/auth/oauth/exchange { code }
    API->>DB: Atomically consume handoff and create session
    API-->>BFF: Return exchange response with user and tokens
    BFF-->>User: Set HttpOnly session cookie and redirect to dashboard
```

---

## 2. Core Security Invariants

### 2.1 SSRF Defense-in-Depth
Any outbound network connection to an external or custom Identity Provider (issuer, discovery endpoint, authorization URL, token endpoint, userinfo endpoint, JWKS URI) is strictly guarded:
- **DNS Resolution & Verification**: Every domain name is resolved before connection. Resolved IP addresses are checked against blocked ranges:
  - IPv4 loopback (`127.0.0.0/8`), RFC 1918 private (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`).
  - Cloud metadata (`169.254.169.254`) and link-local (`169.254.0.0/16`).
  - CGNAT (`100.64.0.0/10`), unspecified (`0.0.0.0/8`), multicast (`224.0.0.0/4`), broadcast (`255.255.255.255`).
  - IPv6 loopback (`::1`), link-local (`fe80::/10`), unique local (`fc00::/7`), IPv4-mapped IPv6 (`::ffff:x.x.x.x`).
- **No Redirects**: The HTTP client is built with `Policy::none()`. External providers cannot redirect requests to internal infrastructure.
- **DNS Pinning**: The client uses `resolve_to_addrs(host, &pinned_addrs)` to eliminate DNS rebinding (TOCTOU) risks.
- **Self-Hosted Allowlisting**: For intranet IdPs, administrators must explicitly set the hostname in deployment-level `ALLOWED_PRIVATE_HOSTS`. There is no per-provider private-host bypass.

### 2.2 Client Secret Encryption at Rest
Client secrets configured by administrators are encrypted at rest using AES-256-GCM via `CryptoService`.
- Raw client secrets are **never** returned in API responses.
- Admin APIs return `has_client_secret`; the secret itself is never returned.
- Provider updates that omit the `client_secret` field retain existing encrypted secrets without requiring re-entry.

### 2.3 Safe Account Linking (Anti-Account Takeover)
- **No Auto-Linking by Email Alone**: An unauthenticated OAuth callback matching an existing registered user's email will **not** automatically link or log in if the user already exists without that OAuth link. This mitigates account takeover risks from IdPs that do not strictly verify email ownership.
- **Explicit Linking Flow**: Authenticated users can link an additional OAuth provider to their account using `POST /api/v1/auth/oauth/link/{provider_key}`.
- **Unlinking Protection**: Users cannot unlink their last remaining authentication method if they do not have a password configured.

### 2.4 Ephemeral Handoff Code
- Handoff codes expire after 60 seconds.
- Single-use invariant: Exchanging a handoff code immediately marks it `used = true` in SurrealDB. Any replay attempt yields `401 Unauthorized`.
- Tokens are never passed via URL query parameters, fragment identifiers, or localStorage.

### 2.5 TOTP / Multi-Factor Authentication Preservation
- If a user has two-factor authentication (TOTP) enabled, the OAuth callback and exchange flow flags `requires_2fa: true` and issues a 2FA challenge rather than directly minting a full session.


### 2.6 Token response encoding
- Form-encoded token responses are percent-decoded as bytes and must produce valid UTF-8.
- Invalid UTF-8 is rejected as a malformed provider response; malformed percent escapes remain literal.

---

## 3. Provider Configuration Guide

### 3.1 Pre-configured Well-Known Providers

#### Google
- **Provider Key**: `google`
- **Default Scopes**: `openid`, `email`, `profile`
- **Endpoints**: Uses standard Google OIDC endpoints.

#### GitHub
- **Provider Key**: `github`
- **Default Scopes**: `read:user`, `user:email`
- **Email Resolution**: Due to GitHub users frequently keeping their public email private, Mensetsu automatically queries `/user/emails` using the token to resolve the primary verified email address.

#### Facebook
- **Provider Key**: `facebook`
- **Default Scopes**: `email`, `public_profile`
- **Profile Resolution**: Automatically queries Graph API `v19.0/me?fields=id,name,email`.

### 3.2 Custom OIDC & OAuth2 Providers
Custom providers (e.g. Okta, Keycloak, Auth0, GitLab) can be added via the Admin API:
- **OIDC Discovery**: If `issuer` is provided (e.g. `https://auth.company.com/realms/main`), Mensetsu auto-discovers endpoints from `/.well-known/openid-configuration`, validating each endpoint against SSRF rules.
- **Manual Endpoints**: Alternatively, manually specify `authorization_endpoint`, `token_endpoint`, `userinfo_endpoint`, and `jwks_uri`.

---

## 4. API Endpoints

### 4.1 Public & User Endpoints
- `GET /api/v1/auth/oauth/providers`: Lists enabled providers for the login UI.
- `GET /api/v1/auth/oauth/authorize/{provider_key}`: Initiates authorization and responds with a 303 redirect to the IdP; the API stores state and sets a short-lived state cookie.
- `GET /api/v1/auth/oauth/callback`: Handles the IdP callback, consumes state, performs code exchange, and redirects to the configured web callback with a one-use `code`.
- `POST /api/v1/auth/oauth/exchange`: Exchanges JSON `{ "code": "..." }` for the session response.
- `GET /api/v1/auth/oauth/accounts`: Lists linked identities for the authenticated user.
- `POST /api/v1/auth/oauth/link/{provider_key}`: Initiates linking for an authenticated user.
- `DELETE /api/v1/auth/oauth/accounts/{identity_id}`: Unlinks a specific identity.

### 4.2 Administrator Endpoints (Admin Role Required)
- `GET /api/v1/admin/oauth/providers`: Lists configured providers and secret presence.
- `PUT /api/v1/admin/oauth/providers/{provider_key}`: Creates or updates a provider; omitted fields are preserved.
- `DELETE /api/v1/admin/oauth/providers/{provider_key}`: Deletes a provider.

Configure each IdP’s registered redirect URI as the API callback URL. Set `OAUTH_WEB_CALLBACK_URL` to the Nuxt `/auth/callback` route; this is the API’s post-login destination, not the IdP redirect URI.

---

## 5. Nuxt BFF Implementation Notes

The Nuxt server routes use the generated contract types and keep tokens server-side. On callback, the BFF reads the one-use `code` from the API redirect, calls the Rust exchange endpoint, stores the returned credentials in secure HttpOnly cookies, and redirects to the sanitized local destination. The API callback is configured as the IdP redirect URI; `OAUTH_WEB_CALLBACK_URL` identifies the Nuxt callback.
