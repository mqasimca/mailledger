//! Implementation for the not-authenticated state.

use std::marker::PhantomData;

use tokio::io::{AsyncRead, AsyncWrite};

use super::Client;
use super::states::{Authenticated, NotAuthenticated};
use crate::command::{Command, TagGenerator};
use crate::connection::framed::FramedStream;
use crate::parser::{Response, ResponseParser, UntaggedResponse};
use crate::types::ResponseCode;
use crate::{Error, Result};
use mailledger_oauth::Token;
use mailledger_oauth::sasl::{oauthbearer_response, xoauth2_response};

impl<S> Client<S, NotAuthenticated>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Creates a new client from a connected stream.
    ///
    /// Reads the server greeting and initial capabilities.
    pub async fn from_stream(stream: S) -> Result<Self> {
        let mut framed = FramedStream::new(stream);

        // Read server greeting
        let greeting = framed.read_response().await?;
        let response = ResponseParser::parse(&greeting)?;

        // Extract capabilities from greeting if present
        let mut capabilities = Vec::new();
        if let Response::Untagged(untagged) = response {
            match untagged {
                UntaggedResponse::Ok {
                    code: Some(ResponseCode::Capability(caps)),
                    ..
                }
                | UntaggedResponse::PreAuth {
                    code: Some(ResponseCode::Capability(caps)),
                    ..
                } => {
                    capabilities = caps;
                }
                UntaggedResponse::Bye { text, .. } => {
                    return Err(Error::Bye(text));
                }
                _ => {}
            }
        }

        Ok(Self {
            stream: framed,
            tag_gen: TagGenerator::default(),
            capabilities,
            _state: PhantomData,
        })
    }

    /// Authenticates with the server using LOGIN.
    ///
    /// Consumes self and returns an authenticated client on success.
    pub async fn login(
        mut self,
        username: &str,
        password: &str,
    ) -> Result<Client<S, Authenticated>> {
        let tag = self.tag_gen.next();
        let cmd = Command::Login {
            username: username.to_string(),
            password: password.to_string(),
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;

        // Update capabilities if included in response
        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Capability(caps))) =
                ResponseParser::parse(response_bytes)
            {
                self.capabilities = caps;
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;

        Ok(Client {
            stream: self.stream,
            tag_gen: self.tag_gen,
            capabilities: self.capabilities,
            _state: PhantomData,
        })
    }

    /// Authenticates with the server using `OAuth2` XOAUTH2 mechanism.
    ///
    /// Consumes self and returns an authenticated client on success.
    /// Uses the XOAUTH2 SASL mechanism (Google/Microsoft proprietary).
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails or if the server doesn't support XOAUTH2.
    pub async fn authenticate_xoauth2(
        mut self,
        email: &str,
        token: &Token,
    ) -> Result<Client<S, Authenticated>> {
        let auth_string = xoauth2_response(email, &token.access_token);
        let tag = self.tag_gen.next();
        let cmd = Command::Authenticate {
            mechanism: "XOAUTH2".to_string(),
            initial_response: Some(auth_string),
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;
        let responses = self.read_until_tagged(&tag).await?;

        // Update capabilities if included in response
        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Capability(caps))) =
                ResponseParser::parse(response_bytes)
            {
                self.capabilities = caps;
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;

        Ok(Client {
            stream: self.stream,
            tag_gen: self.tag_gen,
            capabilities: self.capabilities,
            _state: PhantomData,
        })
    }

    /// Authenticates with the server using `OAuth2` OAUTHBEARER mechanism.
    ///
    /// Consumes self and returns an authenticated client on success.
    /// Uses the OAUTHBEARER SASL mechanism (RFC 7628 standard).
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails or if the server doesn't support OAUTHBEARER.
    pub async fn authenticate_oauthbearer(
        mut self,
        email: &str,
        token: &Token,
    ) -> Result<Client<S, Authenticated>> {
        let auth_string = oauthbearer_response(email, &token.access_token);
        let tag = self.tag_gen.next();
        let cmd = Command::Authenticate {
            mechanism: "OAUTHBEARER".to_string(),
            initial_response: Some(auth_string),
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;
        let responses = self.read_until_tagged(&tag).await?;

        // Update capabilities if included in response
        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Capability(caps))) =
                ResponseParser::parse(response_bytes)
            {
                self.capabilities = caps;
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;

        Ok(Client {
            stream: self.stream,
            tag_gen: self.tag_gen,
            capabilities: self.capabilities,
            _state: PhantomData,
        })
    }

    /// Gracefully disconnects from the server.
    pub async fn logout(mut self) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Logout.serialize(&tag);
        self.stream.write_command(&cmd).await?;

        // Read until we get the tagged response or BYE
        let _ = self.read_until_tagged(&tag).await;

        Ok(())
    }
}
