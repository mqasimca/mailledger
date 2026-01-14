//! Authorization Code Flow implementation.

use super::{OAuthClient, PkceChallenge};
use crate::error::Result;
use crate::token::Token;
use url::Url;

/// Authorization Code Flow for `OAuth2`.
///
/// This flow is suitable for applications that can open a browser
/// and receive the authorization code via redirect.
#[derive(Debug)]
pub struct AuthorizationCodeFlow {
    client: OAuthClient,
    pkce: Option<PkceChallenge>,
}

impl AuthorizationCodeFlow {
    /// Creates a new authorization code flow.
    #[must_use]
    pub const fn new(client: OAuthClient) -> Self {
        Self { client, pkce: None }
    }

    /// Enables PKCE for enhanced security (recommended for public clients).
    #[must_use]
    pub fn with_pkce(mut self) -> Self {
        self.pkce = Some(PkceChallenge::generate());
        self
    }

    /// Builds the authorization URL for user consent.
    ///
    /// The user should be redirected to this URL to authorize the application.
    ///
    /// # Arguments
    ///
    /// * `scopes` - Optional scopes to request (uses provider defaults if None)
    /// * `state` - Optional state parameter for CSRF protection
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be constructed.
    pub fn authorization_url(&self, scopes: Option<&[String]>, state: Option<&str>) -> Result<Url> {
        let mut url = self.client.provider.auth_url.clone();

        {
            let mut pairs = url.query_pairs_mut();
            pairs
                .append_pair("client_id", &self.client.client_id)
                .append_pair("response_type", "code");

            if let Some(redirect_uri) = &self.client.redirect_uri {
                pairs.append_pair("redirect_uri", redirect_uri);
            }

            let scope_str = scopes.map_or_else(
                || self.client.provider.default_scopes.join(" "),
                |s| s.join(" "),
            );

            if !scope_str.is_empty() {
                pairs.append_pair("scope", &scope_str);
            }

            if let Some(state_val) = state {
                pairs.append_pair("state", state_val);
            }

            if let Some(pkce) = &self.pkce {
                pairs
                    .append_pair("code_challenge", pkce.challenge())
                    .append_pair("code_challenge_method", pkce.method());
            }

            // Provider-specific parameters
            match self.client.provider.name.as_str() {
                "Google" => {
                    pairs
                        .append_pair("access_type", "offline")
                        .append_pair("prompt", "consent");
                }
                "Microsoft" => {
                    pairs.append_pair("prompt", "consent");
                }
                _ => {}
            }
        }

        Ok(url)
    }

    /// Exchanges the authorization code for an access token.
    ///
    /// # Arguments
    ///
    /// * `code` - Authorization code from the redirect
    /// * `redirect_uri` - Optional redirect URI (uses client config if None)
    ///
    /// # Errors
    ///
    /// Returns an error if the token exchange fails.
    pub async fn exchange_code(&self, code: &str, redirect_uri: Option<&str>) -> Result<Token> {
        let code_verifier = self.pkce.as_ref().map(PkceChallenge::verifier);
        self.client
            .exchange_code(code, redirect_uri, code_verifier)
            .await
    }

    /// Returns the PKCE verifier if PKCE is enabled.
    #[must_use]
    pub fn pkce_verifier(&self) -> Option<&str> {
        self.pkce.as_ref().map(PkceChallenge::verifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::Provider;

    #[test]
    fn test_authorization_url() {
        let provider = Provider::google().unwrap();
        let client =
            OAuthClient::new("test_client", provider).with_redirect_uri("http://localhost:8080");

        let flow = AuthorizationCodeFlow::new(client);
        let url = flow.authorization_url(None, Some("random_state")).unwrap();

        assert!(url.as_str().contains("client_id=test_client"));
        assert!(url.as_str().contains("response_type=code"));
        assert!(url.as_str().contains("state=random_state"));
        // Check URL-encoded redirect_uri
        assert!(
            url.as_str()
                .contains("redirect_uri=http%3A%2F%2Flocalhost%3A8080")
        );
    }

    #[test]
    fn test_authorization_url_with_pkce() {
        let provider = Provider::google().unwrap();
        let client = OAuthClient::new("test_client", provider);

        let flow = AuthorizationCodeFlow::new(client).with_pkce();
        let url = flow.authorization_url(None, None).unwrap();

        assert!(url.as_str().contains("code_challenge="));
        assert!(url.as_str().contains("code_challenge_method=S256"));
        assert!(flow.pkce_verifier().is_some());
    }

    #[test]
    fn test_authorization_url_custom_scopes() {
        let provider = Provider::google().unwrap();
        let client = OAuthClient::new("test_client", provider);

        let flow = AuthorizationCodeFlow::new(client);
        let scopes = vec!["email".to_string(), "profile".to_string()];
        let url = flow.authorization_url(Some(&scopes), None).unwrap();

        // Check URL-encoded scope (space becomes + in query parameters)
        assert!(url.as_str().contains("scope=email+profile"));
    }

    #[test]
    fn test_google_specific_params() {
        let provider = Provider::google().unwrap();
        let client = OAuthClient::new("test_client", provider);

        let flow = AuthorizationCodeFlow::new(client);
        let url = flow.authorization_url(None, None).unwrap();

        assert!(url.as_str().contains("access_type=offline"));
        assert!(url.as_str().contains("prompt=consent"));
    }
}
