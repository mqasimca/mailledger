//! Device Authorization Flow implementation (RFC 8628).

use super::OAuthClient;
use crate::error::{Error, Result};
use crate::token::{ErrorResponse, Token, TokenResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Device authorization response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceAuthorization {
    /// Device code for polling.
    pub device_code: String,
    /// User code to display to the user.
    pub user_code: String,
    /// Verification URI where user should go.
    pub verification_uri: String,
    /// Complete verification URI (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_uri_complete: Option<String>,
    /// Expiration time in seconds.
    pub expires_in: u32,
    /// Polling interval in seconds (minimum 5 seconds).
    #[serde(default = "default_interval")]
    pub interval: u32,
}

const fn default_interval() -> u32 {
    5
}

/// Device Authorization Flow for `OAuth2`.
///
/// This flow is suitable for devices with limited input capabilities
/// or no browser (e.g., CLI applications, `IoT` devices).
#[derive(Debug)]
pub struct DeviceFlow {
    client: OAuthClient,
}

impl DeviceFlow {
    /// Creates a new device flow.
    #[must_use]
    pub const fn new(client: OAuthClient) -> Self {
        Self { client }
    }

    /// Requests device authorization from the server.
    ///
    /// Returns the device code and user code that should be displayed to the user.
    ///
    /// # Arguments
    ///
    /// * `scopes` - Optional scopes to request (uses provider defaults if None)
    ///
    /// # Errors
    ///
    /// Returns an error if the authorization request fails.
    pub async fn request_device_authorization(
        &self,
        scopes: Option<&[String]>,
    ) -> Result<DeviceAuthorization> {
        let device_auth_url = self
            .client
            .provider
            .device_auth_url
            .as_ref()
            .ok_or_else(|| {
                Error::InvalidConfig(format!(
                    "Provider {} does not support device flow",
                    self.client.provider.name
                ))
            })?;

        let scope_str = scopes.map_or_else(
            || self.client.provider.default_scopes.join(" "),
            |s| s.join(" "),
        );

        let mut params = HashMap::new();
        params.insert("client_id", self.client.client_id.as_str());
        if !scope_str.is_empty() {
            params.insert("scope", &scope_str);
        }

        let response = self
            .client
            .http_client
            .post(device_auth_url.clone())
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;
            return Err(error.into_error());
        }

        response.json().await.map_err(Into::into)
    }

    /// Polls for token completion.
    ///
    /// This should be called repeatedly after displaying the user code
    /// until the user completes authorization or the device code expires.
    ///
    /// # Arguments
    ///
    /// * `device_code` - Device code from authorization request
    /// * `interval` - Polling interval in seconds (from authorization response)
    ///
    /// # Errors
    ///
    /// Returns an error if polling fails. Returns `Error::AccessDenied` if user
    /// denies authorization. Returns specific errors for `authorization_pending`
    /// and `slow_down` which should be handled by continuing to poll.
    pub async fn poll_for_token(&self, device_code: &str, interval: Duration) -> Result<Token> {
        tokio::time::sleep(interval).await;

        let mut params = HashMap::new();
        params.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");
        params.insert("device_code", device_code);
        params.insert("client_id", &self.client.client_id);

        let response = self
            .client
            .http_client
            .post(self.client.provider.token_url.clone())
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().await?;

            // Handle special device flow error codes
            return match error.error.as_str() {
                "authorization_pending" => Err(Error::oauth_error(
                    "authorization_pending",
                    "User has not yet authorized",
                )),
                "slow_down" => Err(Error::oauth_error(
                    "slow_down",
                    "Polling too frequently, slow down",
                )),
                "access_denied" => Err(Error::AccessDenied),
                "expired_token" => Err(Error::TokenExpired),
                _ => Err(error.into_error()),
            };
        }

        let token_response: TokenResponse = response.json().await?;
        Token::from_response(token_response)
    }

    /// Complete device authorization flow.
    ///
    /// This is a convenience method that:
    /// 1. Requests device authorization
    /// 2. Returns the user code and verification URI (caller should display these)
    /// 3. Polls for token completion with automatic retry
    ///
    /// # Arguments
    ///
    /// * `scopes` - Optional scopes to request
    /// * `max_attempts` - Maximum number of polling attempts (0 = unlimited)
    ///
    /// # Errors
    ///
    /// Returns an error if authorization fails or times out.
    pub async fn authorize(
        &self,
        scopes: Option<&[String]>,
        max_attempts: usize,
    ) -> Result<(DeviceAuthorization, Token)> {
        let auth = self.request_device_authorization(scopes).await?;

        let mut interval = Duration::from_secs(u64::from(auth.interval));
        let mut attempts = 0;

        loop {
            if max_attempts > 0 && attempts >= max_attempts {
                return Err(Error::Timeout(auth.expires_in.into()));
            }

            match self.poll_for_token(&auth.device_code, interval).await {
                Ok(token) => return Ok((auth, token)),
                Err(Error::OAuth { ref error, .. }) if error == "authorization_pending" => {
                    attempts += 1;
                }
                Err(Error::OAuth { ref error, .. }) if error == "slow_down" => {
                    // Increase interval by 5 seconds as per RFC
                    interval += Duration::from_secs(5);
                    attempts += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::Provider;

    #[test]
    fn test_device_flow_creation() {
        let provider = Provider::google().unwrap();
        let client = OAuthClient::new("test_client", provider);
        let _flow = DeviceFlow::new(client);
    }

    #[test]
    fn test_default_interval() {
        assert_eq!(default_interval(), 5);
    }

    #[test]
    fn test_device_auth_deserialization() {
        let json = r#"{
            "device_code": "dev123",
            "user_code": "USER-CODE",
            "verification_uri": "https://example.com/device",
            "expires_in": 1800,
            "interval": 5
        }"#;

        let auth: DeviceAuthorization = serde_json::from_str(json).unwrap();
        assert_eq!(auth.device_code, "dev123");
        assert_eq!(auth.user_code, "USER-CODE");
        assert_eq!(auth.interval, 5);
    }
}
