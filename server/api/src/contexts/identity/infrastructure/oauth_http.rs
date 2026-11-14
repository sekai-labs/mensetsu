use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use reqwest::{Client, Url};
use serde::Deserialize;

use crate::contexts::identity::domain::oauth::{
    normalize_issuer, validate_id_token_claims, IdTokenClaims, OAuthProviderConfig,
    ValidatedIdToken,
};
use crate::shared::error::{AppError, Result};
use crate::shared::ssrf::{build_safe_client, resolve_and_verify_url};

const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const REQUEST_TIMEOUT_SECONDS: u64 = 10;
const USER_AGENT_VALUE: &str = "mensetsu-oauth/1.0";
const ALLOWED_ID_TOKEN_ALGORITHMS: &[Algorithm] = &[
    Algorithm::RS256,
    Algorithm::RS384,
    Algorithm::RS512,
    Algorithm::ES256,
    Algorithm::ES384,
    Algorithm::EdDSA,
];

/// Validates server-fetched OAuth endpoints against SSRF rules. Kept as a
/// trait so provider configuration logic is testable without live DNS.
#[async_trait]
pub trait EndpointValidator: Send + Sync {
    async fn validate_endpoint(&self, url: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct ResolvedOAuthEndpoints {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub email_endpoint: Option<String>,
    pub jwks_uri: Option<String>,
    pub issuer: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OidcDiscoveryDocument {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    userinfo_endpoint: Option<String>,
    jwks_uri: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenEndpointResponse {
    pub access_token: String,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Clone)]
pub struct OAuthHttpClient {
    allowlist: Vec<String>,
}

impl OAuthHttpClient {
    pub fn new(allowlist: Vec<String>) -> Self {
        Self { allowlist }
    }

    fn client_for(&self, url: &Url, addresses: &[std::net::SocketAddr]) -> Result<Client> {
        let host = url
            .host_str()
            .ok_or_else(|| AppError::Forbidden("OAuth endpoint has no host".into()))?;
        build_safe_client(host, addresses)
    }

    async fn fetch_json(&self, raw_url: &str) -> Result<serde_json::Value> {
        let (url, addresses) = resolve_and_verify_url(raw_url, true, &self.allowlist).await?;
        let client = self.client_for(&url, &addresses)?;
        let response = client
            .get(url)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalService(format!("OAuth endpoint request failed: {e}"))
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(AppError::ExternalService(format!(
                "OAuth endpoint returned HTTP {}",
                status.as_u16()
            )));
        }
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let body = self.read_limited(response).await?;

        if !content_type.contains("json") {
            return Err(AppError::ExternalService(
                "OAuth endpoint did not return JSON".into(),
            ));
        }
        serde_json::from_slice(&body)
            .map_err(|_| AppError::ExternalService("OAuth endpoint returned invalid JSON".into()))
    }

    async fn post_form_json(
        &self,
        raw_url: &str,
        form: &[(String, String)],
        bearer: Option<&str>,
    ) -> Result<serde_json::Value> {
        let (url, addresses) = resolve_and_verify_url(raw_url, true, &self.allowlist).await?;
        let client = self.client_for(&url, &addresses)?;
        let mut request = client
            .post(url)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(ACCEPT, "application/json")
            .form(form);
        if let Some(token) = bearer {
            request = request.bearer_auth(token);
        }
        let response = request
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("OAuth token request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            return Err(AppError::ExternalService(format!(
                "OAuth token endpoint returned HTTP {}",
                status.as_u16()
            )));
        }
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let body = self.read_limited(response).await?;

        if content_type.contains("json") || body.first() == Some(&b'{') {
            serde_json::from_slice(&body)
                .map_err(|_| AppError::ExternalService("OAuth token response was invalid".into()))
        } else if content_type.contains("application/x-www-form-urlencoded") {
            let text = String::from_utf8(body).map_err(|_| {
                AppError::ExternalService("OAuth token response was invalid".into())
            })?;
            let mut map = serde_json::Map::new();
            for (key, value) in parse_form_body(&text)? {
                map.insert(key, serde_json::Value::String(value));
            }
            Ok(serde_json::Value::Object(map))
        } else {
            Err(AppError::ExternalService(
                "OAuth token response was not JSON or form encoded".into(),
            ))
        }
    }

    async fn get_userinfo_json(
        &self,
        raw_url: &str,
        access_token: &str,
        extra_accept: &str,
    ) -> Result<serde_json::Value> {
        let (url, addresses) = resolve_and_verify_url(raw_url, true, &self.allowlist).await?;
        let client = self.client_for(&url, &addresses)?;
        let response = client
            .get(url)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECONDS))
            .header(USER_AGENT, USER_AGENT_VALUE)
            .header(ACCEPT, extra_accept)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                AppError::ExternalService(format!("OAuth userinfo request failed: {e}"))
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(AppError::ExternalService(format!(
                "OAuth userinfo endpoint returned HTTP {}",
                status.as_u16()
            )));
        }
        let body = self.read_limited(response).await?;
        serde_json::from_slice(&body)
            .map_err(|_| AppError::ExternalService("OAuth userinfo returned invalid JSON".into()))
    }

    async fn read_limited(&self, response: reqwest::Response) -> Result<Vec<u8>> {
        let body = response
            .bytes()
            .await
            .map_err(|e| AppError::ExternalService(format!("OAuth response read failed: {e}")))?;
        if body.len() > MAX_RESPONSE_BYTES {
            return Err(AppError::ExternalService(
                "OAuth response exceeded the maximum size".into(),
            ));
        }
        Ok(body.to_vec())
    }

    /// Discovers OIDC metadata from an issuer and re-validates every endpoint
    /// returned by the discovery document. Discovery-supplied URLs are never
    /// trusted merely because the issuer was validated.
    pub async fn discover(&self, issuer: &str) -> Result<ResolvedOAuthEndpoints> {
        let normalized_issuer = normalize_issuer(issuer);
        if normalized_issuer.is_empty() {
            return Err(AppError::Validation("OIDC issuer is required".into()));
        }
        EndpointValidator::validate_endpoint(self, &normalized_issuer).await?;

        let discovery_url = format!("{normalized_issuer}/.well-known/openid-configuration");
        let document: OidcDiscoveryDocument =
            serde_json::from_value(self.fetch_json(&discovery_url).await?).map_err(|_| {
                AppError::ExternalService(
                    "OIDC discovery document is missing required fields".into(),
                )
            })?;

        if normalize_issuer(&document.issuer) != normalized_issuer {
            return Err(AppError::ExternalService(
                "OIDC discovery issuer does not match the configured issuer".into(),
            ));
        }

        let userinfo_endpoint = document.userinfo_endpoint.ok_or_else(|| {
            AppError::ExternalService("OIDC discovery is missing userinfo_endpoint".into())
        })?;
        let jwks_uri = document.jwks_uri.ok_or_else(|| {
            AppError::ExternalService("OIDC discovery is missing jwks_uri".into())
        })?;

        for (name, endpoint) in [
            ("authorization_endpoint", &document.authorization_endpoint),
            ("token_endpoint", &document.token_endpoint),
            ("userinfo_endpoint", &userinfo_endpoint),
            ("jwks_uri", &jwks_uri),
        ] {
            EndpointValidator::validate_endpoint(self, endpoint)
                .await
                .map_err(|error| {
                    AppError::ExternalService(format!(
                        "OIDC discovery returned an unsafe {name}: {error}"
                    ))
                })?;
        }

        Ok(ResolvedOAuthEndpoints {
            authorization_endpoint: document.authorization_endpoint,
            token_endpoint: document.token_endpoint,
            userinfo_endpoint,
            email_endpoint: None,
            jwks_uri: Some(jwks_uri),
            issuer: Some(normalized_issuer),
        })
    }

    /// Resolves the endpoints used for a provider. OIDC providers use live
    /// discovery; manual OAuth2 providers use admin-configured endpoints, all
    /// re-validated against SSRF rules before use.
    pub async fn resolve_endpoints(
        &self,
        provider: &OAuthProviderConfig,
    ) -> Result<ResolvedOAuthEndpoints> {
        if provider.is_oidc() {
            let issuer = provider
                .issuer
                .as_deref()
                .or_else(|| provider.provider_type.default_issuer())
                .ok_or_else(|| AppError::Validation("OIDC issuer is not configured".into()))?;
            return self.discover(issuer).await;
        }

        let authorization_endpoint = provider
            .authorization_endpoint
            .clone()
            .or_else(|| {
                provider
                    .provider_type
                    .default_authorization_endpoint()
                    .map(str::to_string)
            })
            .ok_or_else(|| AppError::Validation("authorization_endpoint is required".into()))?;
        let token_endpoint = provider
            .token_endpoint
            .clone()
            .or_else(|| {
                provider
                    .provider_type
                    .default_token_endpoint()
                    .map(str::to_string)
            })
            .ok_or_else(|| AppError::Validation("token_endpoint is required".into()))?;
        let userinfo_endpoint = provider
            .userinfo_endpoint
            .clone()
            .or_else(|| {
                provider
                    .provider_type
                    .default_userinfo_endpoint()
                    .map(str::to_string)
            })
            .ok_or_else(|| AppError::Validation("userinfo_endpoint is required".into()))?;
        let email_endpoint = provider.email_endpoint.clone().or_else(|| {
            provider
                .provider_type
                .default_email_endpoint()
                .map(str::to_string)
        });

        for endpoint in [&authorization_endpoint, &token_endpoint, &userinfo_endpoint] {
            EndpointValidator::validate_endpoint(self, endpoint).await?;
        }
        if let Some(email_endpoint) = &email_endpoint {
            EndpointValidator::validate_endpoint(self, email_endpoint).await?;
        }

        Ok(ResolvedOAuthEndpoints {
            authorization_endpoint,
            token_endpoint,
            userinfo_endpoint,
            email_endpoint,
            jwks_uri: provider.jwks_uri.clone(),
            issuer: provider.issuer.clone(),
        })
    }

    pub async fn exchange_code(
        &self,
        provider: &OAuthProviderConfig,
        client_secret: &str,
        endpoints: &ResolvedOAuthEndpoints,
        code: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> Result<TokenEndpointResponse> {
        let form = vec![
            ("grant_type".to_string(), "authorization_code".to_string()),
            ("code".to_string(), code.to_string()),
            ("redirect_uri".to_string(), redirect_uri.to_string()),
            ("client_id".to_string(), provider.client_id.clone()),
            ("client_secret".to_string(), client_secret.to_string()),
            ("code_verifier".to_string(), code_verifier.to_string()),
        ];

        let value = self
            .post_form_json(&endpoints.token_endpoint, &form, None)
            .await?;

        if let Some(error) = value.get("error").and_then(|value| value.as_str()) {
            let code = match error {
                "invalid_grant" => "invalid_grant",
                "invalid_client" => "invalid_client",
                "access_denied" => "access_denied",
                _ => "token_exchange_failed",
            };
            return Err(AppError::ExternalService(format!(
                "OAuth token exchange was rejected ({code})"
            )));
        }

        let value = match value {
            serde_json::Value::Object(map) => serde_json::Value::Object(map),
            _ => {
                return Err(AppError::ExternalService(
                    "OAuth token response was not an object".into(),
                ))
            }
        };
        let response: TokenEndpointResponse = serde_json::from_value(value)
            .map_err(|_| AppError::ExternalService("OAuth token response was invalid".into()))?;
        if response.access_token.trim().is_empty() {
            return Err(AppError::ExternalService(
                "OAuth token response was missing an access token".into(),
            ));
        }
        Ok(response)
    }

    pub async fn fetch_userinfo(
        &self,
        provider: &OAuthProviderConfig,
        endpoints: &ResolvedOAuthEndpoints,
        access_token: &str,
    ) -> Result<serde_json::Value> {
        let url = match provider.provider_type {
            crate::contexts::identity::domain::oauth::OAuthProviderType::Facebook => {
                let separator = if endpoints.userinfo_endpoint.contains('?') {
                    '&'
                } else {
                    '?'
                };
                format!(
                    "{}{separator}fields=id,name,email",
                    endpoints.userinfo_endpoint
                )
            }
            _ => endpoints.userinfo_endpoint.clone(),
        };
        let accept = match provider.provider_type {
            crate::contexts::identity::domain::oauth::OAuthProviderType::Github => {
                "application/vnd.github+json"
            }
            _ => "application/json",
        };
        self.get_userinfo_json(&url, access_token, accept).await
    }

    pub async fn fetch_github_emails(
        &self,
        endpoints: &ResolvedOAuthEndpoints,
        access_token: &str,
    ) -> Result<Vec<serde_json::Value>> {
        let endpoint = endpoints.email_endpoint.as_deref().ok_or_else(|| {
            AppError::Validation("GitHub email_endpoint is not configured".into())
        })?;
        let value = self
            .get_userinfo_json(endpoint, access_token, "application/vnd.github+json")
            .await?;
        value
            .as_array()
            .cloned()
            .ok_or_else(|| AppError::ExternalService("GitHub email response was invalid".into()))
    }

    pub async fn validate_id_token(
        &self,
        provider: &OAuthProviderConfig,
        endpoints: &ResolvedOAuthEndpoints,
        token: &str,
        expected_nonce: &str,
    ) -> Result<ValidatedIdToken> {
        let issuer = endpoints
            .issuer
            .as_deref()
            .or(provider.issuer.as_deref())
            .ok_or_else(|| AppError::Validation("OIDC issuer is not configured".into()))?;
        let jwks_uri = endpoints
            .jwks_uri
            .as_deref()
            .ok_or_else(|| AppError::Validation("OIDC jwks_uri is not configured".into()))?;

        let header = decode_header(token)
            .map_err(|_| AppError::Unauthorized("OIDC ID token header is invalid".into()))?;
        if !ALLOWED_ID_TOKEN_ALGORITHMS.contains(&header.alg) {
            return Err(AppError::Unauthorized(
                "OIDC ID token uses a disallowed algorithm".into(),
            ));
        }

        let jwks_value = self.fetch_json(jwks_uri).await?;
        let jwks: JwkSet = serde_json::from_value(jwks_value)
            .map_err(|_| AppError::ExternalService("OIDC JWKS was invalid".into()))?;

        let jwk = match header.kid.as_deref() {
            Some(kid) => jwks
                .keys
                .iter()
                .find(|jwk| jwk.common.key_id.as_deref() == Some(kid)),
            None if jwks.keys.len() == 1 => jwks.keys.first(),
            None => None,
        }
        .ok_or_else(|| AppError::Unauthorized("OIDC signing key was not found".into()))?;

        let decoding_key = DecodingKey::from_jwk(jwk)
            .map_err(|_| AppError::Unauthorized("OIDC signing key is invalid".into()))?;
        let mut validation = Validation::new(header.alg);
        validation.set_audience(&[provider.client_id.as_str()]);
        validation.set_issuer(&[issuer]);
        validation.validate_nbf = false;
        validation.leeway = 60;

        let token_data = decode::<IdTokenClaims>(token, &decoding_key, &validation)
            .map_err(|_| AppError::Unauthorized("OIDC ID token verification failed".into()))?;

        validate_id_token_claims(
            &token_data.claims,
            issuer,
            &provider.client_id,
            expected_nonce,
            Utc::now(),
            60,
        )
    }
}

#[async_trait]
impl EndpointValidator for OAuthHttpClient {
    async fn validate_endpoint(&self, url: &str) -> Result<()> {
        let _ = resolve_and_verify_url(url, true, &self.allowlist).await?;
        Ok(())
    }
}

fn parse_form_body(body: &str) -> Result<HashMap<String, String>> {
    let mut pairs = HashMap::new();
    for pair in body.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        pairs.insert(percent_decode(key)?, percent_decode(value)?);
    }
    Ok(pairs)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn percent_decode(raw: &str) -> Result<String> {
    let bytes = raw.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                match (hex_nibble(bytes[index + 1]), hex_nibble(bytes[index + 2])) {
                    (Some(high), Some(low)) => {
                        output.push((high << 4) | low);
                        index += 3;
                    }
                    _ => {
                        output.push(b'%');
                        index += 1;
                    }
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(output)
        .map_err(|_| AppError::ExternalService("OAuth token response was invalid".into()))
}

#[cfg(test)]
mod form_encoding_tests {
    use super::{parse_form_body, percent_decode};

    #[test]
    fn decodes_plus_percent_escapes_and_utf8() {
        assert_eq!(percent_decode("a+b%20c%2Fd").unwrap(), "a b c/d");
        assert_eq!(percent_decode("caf%C3%A9").unwrap(), "café");
    }

    #[test]
    fn malformed_percent_escape_with_unicode_does_not_panic() {
        assert_eq!(percent_decode("%€x").unwrap(), "%€x");
        assert_eq!(percent_decode("%").unwrap(), "%");
    }

    #[test]
    fn rejects_decoded_invalid_utf8() {
        assert!(percent_decode("%FF").is_err());
        assert!(parse_form_body("error=%FF").is_err());
        assert_eq!(
            parse_form_body("access_token=abc&token_type=Bearer").unwrap()["access_token"],
            "abc"
        );
    }
}

/// Provider HTTP surface used by the application service. A trait keeps the
/// login/link state machine testable without live provider traffic.
#[async_trait]
pub trait OAuthProviderClient: Send + Sync {
    async fn validate_endpoint(&self, url: &str) -> Result<()>;
    async fn resolve_endpoints(
        &self,
        provider: &OAuthProviderConfig,
    ) -> Result<ResolvedOAuthEndpoints>;
    async fn exchange_code(
        &self,
        provider: &OAuthProviderConfig,
        client_secret: &str,
        endpoints: &ResolvedOAuthEndpoints,
        code: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> Result<TokenEndpointResponse>;
    async fn fetch_userinfo(
        &self,
        provider: &OAuthProviderConfig,
        endpoints: &ResolvedOAuthEndpoints,
        access_token: &str,
    ) -> Result<serde_json::Value>;
    async fn fetch_github_emails(
        &self,
        endpoints: &ResolvedOAuthEndpoints,
        access_token: &str,
    ) -> Result<Vec<serde_json::Value>>;
    async fn validate_id_token(
        &self,
        provider: &OAuthProviderConfig,
        endpoints: &ResolvedOAuthEndpoints,
        token: &str,
        expected_nonce: &str,
    ) -> Result<ValidatedIdToken>;
}

#[async_trait]
impl OAuthProviderClient for OAuthHttpClient {
    async fn validate_endpoint(&self, url: &str) -> Result<()> {
        <Self as EndpointValidator>::validate_endpoint(self, url).await
    }

    async fn resolve_endpoints(
        &self,
        provider: &OAuthProviderConfig,
    ) -> Result<ResolvedOAuthEndpoints> {
        OAuthHttpClient::resolve_endpoints(self, provider).await
    }

    async fn exchange_code(
        &self,
        provider: &OAuthProviderConfig,
        client_secret: &str,
        endpoints: &ResolvedOAuthEndpoints,
        code: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> Result<TokenEndpointResponse> {
        OAuthHttpClient::exchange_code(
            self,
            provider,
            client_secret,
            endpoints,
            code,
            code_verifier,
            redirect_uri,
        )
        .await
    }

    async fn fetch_userinfo(
        &self,
        provider: &OAuthProviderConfig,
        endpoints: &ResolvedOAuthEndpoints,
        access_token: &str,
    ) -> Result<serde_json::Value> {
        OAuthHttpClient::fetch_userinfo(self, provider, endpoints, access_token).await
    }

    async fn fetch_github_emails(
        &self,
        endpoints: &ResolvedOAuthEndpoints,
        access_token: &str,
    ) -> Result<Vec<serde_json::Value>> {
        OAuthHttpClient::fetch_github_emails(self, endpoints, access_token).await
    }

    async fn validate_id_token(
        &self,
        provider: &OAuthProviderConfig,
        endpoints: &ResolvedOAuthEndpoints,
        token: &str,
        expected_nonce: &str,
    ) -> Result<ValidatedIdToken> {
        OAuthHttpClient::validate_id_token(self, provider, endpoints, token, expected_nonce).await
    }
}
