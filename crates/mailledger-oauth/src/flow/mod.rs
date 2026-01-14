//! `OAuth2` authorization flows.

mod code;
mod device;
mod pkce;

pub use code::AuthorizationCodeFlow;
pub use device::DeviceFlow;
pub use pkce::PkceChallenge;

use crate::error::Result;
use crate::provider::Provider;
use crate::token::{ErrorResponse, Token, TokenResponse};
use reqwest::Client;
use std::collections::HashMap;

/// Common `OAuth2` client configuration.
#[derive(Debug, Clone)]
pub struct OAuthClient {
    /// Client ID from provider.
    pub client_id: String,
    /// Client secret (optional for public clients).
    pub client_secret: Option<String>,
    /// Redirect URI for authorization code flow.
    pub redirect_uri: Option<String>,
    /// Provider configuration.
    pub provider: Provider,
    /// HTTP client.
    http_client: Client,
}

impl OAuthClient {
    /// Creates a new OAuth client.
    #[must_use]
    pub fn new(client_id: impl Into<String>, provider: Provider) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: None,
            redirect_uri: None,
            provider,
            http_client: Client::new(),
        }
    }

    /// Sets the client secret.
    #[must_use]
    pub fn with_client_secret(mut self, secret: impl Into<String>) -> Self {
        self.client_secret = Some(secret.into());
        self
    }

    /// Sets the redirect URI.
    #[must_use]
    pub fn with_redirect_uri(mut self, uri: impl Into<String>) -> Self {
        self.redirect_uri = Some(uri.into());
        self
    }

    /// Refreshes an access token using a refresh token.
    ///
    /// # Errors
    ///
    /// Returns an error if the refresh fails or if the token has no refresh token.
    pub async fn refresh_token(&self, token: &Token) -> Result<Token> {
        let refresh_token = token.refresh_token()?;

        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", refresh_token);
        params.insert("client_id", &self.client_id);

        if let Some(secret) = &self.client_secret {
            params.insert("client_secret", secret);
        }

        let response = self
            .http_client
            .post(self.provider.token_url.clone())
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(error.into_error());
        }

        let token_response: TokenResponse = response.json().await?;
        let mut new_token = Token::from_response(token_response)?;

        // Preserve refresh token if not returned
        if new_token.refresh_token.is_none() {
            new_token.refresh_token.clone_from(&token.refresh_token);
        }

        Ok(new_token)
    }

    /// Exchanges an authorization code for tokens.
    ///
    /// # Errors
    ///
    /// Returns an error if the exchange fails.
    pub(crate) async fn exchange_code(
        &self,
        code: &str,
        redirect_uri: Option<&str>,
        code_verifier: Option<&str>,
    ) -> Result<Token> {
        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code);
        params.insert("client_id", &self.client_id);

        if let Some(uri) = redirect_uri.or(self.redirect_uri.as_deref()) {
            params.insert("redirect_uri", uri);
        }

        if let Some(secret) = &self.client_secret {
            params.insert("client_secret", secret);
        }

        if let Some(verifier) = code_verifier {
            params.insert("code_verifier", verifier);
        }

        let response = self
            .http_client
            .post(self.provider.token_url.clone())
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(error.into_error());
        }

        let token_response: TokenResponse = response.json().await?;
        Token::from_response(token_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_client_creation() {
        let provider = Provider::google().unwrap();
        let client = OAuthClient::new("test_client_id", provider);
        assert_eq!(client.client_id, "test_client_id");
        assert!(client.client_secret.is_none());
    }

    #[test]
    fn test_oauth_client_with_secret() {
        let provider = Provider::google().unwrap();
        let client = OAuthClient::new("test_client_id", provider)
            .with_client_secret("secret")
            .with_redirect_uri("http://localhost:8080");

        assert_eq!(client.client_secret.as_deref(), Some("secret"));
        assert_eq!(
            client.redirect_uri.as_deref(),
            Some("http://localhost:8080")
        );
    }
}
