use crate::auth::AuthError;
use crate::auth::TokenManager;
use crate::auth::oauth::device::{DeviceFlow, DevicePollOutcome};
use crate::auth::store::TokenStore;
use crate::mcp::{McpError, Tool};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

/// Shared dependencies needed by the auth tools.
pub struct AuthShared {
    pub store: Arc<dyn TokenStore>,
    pub client_id: String,
    pub account: String,
    pub scopes: Vec<String>,
    pub token_manager: Arc<TokenManager>,
}

pub struct GmailBeginAuthTool {
    pub shared: Arc<AuthShared>,
}

#[derive(Deserialize)]
struct BeginAuthArgs {
    #[serde(default)]
    email: Option<String>,
}

#[async_trait]
impl Tool for GmailBeginAuthTool {
    fn name(&self) -> &str {
        "gmail_begin_auth"
    }
    fn description(&self) -> &str {
        "Start interactive OAuth re-authentication when the stored Gmail token \
         is missing, expired, or revoked (errors mentioning 'token was revoked' \
         or 'no credentials'). Returns a verification URL and short user code \
         that the human must enter at that URL. Then call `gmail_complete_auth` \
         with the returned `device_code` (poll repeatedly until status='ok'). \
         Optional `email` pre-fills the consent screen for that Google account."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "email": {
                    "type": "string",
                    "description": "Optional Google account email to pre-fill on the consent screen (login_hint)."
                }
            },
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let parsed: BeginAuthArgs = serde_json::from_value(args)
            .map_err(|e| McpError::InvalidParams(e.to_string()))?;

        let flow = DeviceFlow::new(self.shared.client_id.clone(), self.shared.scopes.clone());
        let init = flow
            .begin(parsed.email.as_deref())
            .await
            .map_err(|e| McpError::ToolError(anyhow::anyhow!(e)))?;

        Ok(serde_json::json!({
            "device_code": init.device_code,
            "user_code": init.user_code,
            "verification_url": init.verification_url,
            "expires_in": init.expires_in,
            "interval": init.interval,
            "next_action": "Show verification_url and user_code to the human. \
                            After they consent, call gmail_complete_auth with the device_code; \
                            keep polling (waiting `interval` seconds between calls) until status='ok'."
        }))
    }
}

pub struct GmailCompleteAuthTool {
    pub shared: Arc<AuthShared>,
}

#[derive(Deserialize)]
struct CompleteAuthArgs {
    device_code: String,
}

#[async_trait]
impl Tool for GmailCompleteAuthTool {
    fn name(&self) -> &str {
        "gmail_complete_auth"
    }
    fn description(&self) -> &str {
        "Poll once to complete OAuth started with `gmail_begin_auth`. \
         Returns status='pending' if the human hasn't consented yet \
         (wait `retry_after_seconds`, then call again with the same device_code), \
         status='ok' on success (token stored, retry the original Gmail call), \
         status='denied' or 'expired' on terminal failure (start over with gmail_begin_auth)."
    }
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "device_code": {
                    "type": "string",
                    "description": "device_code returned by gmail_begin_auth."
                }
            },
            "required": ["device_code"],
            "additionalProperties": false
        })
    }
    async fn call(&self, args: serde_json::Value) -> Result<serde_json::Value, McpError> {
        let parsed: CompleteAuthArgs = serde_json::from_value(args)
            .map_err(|e| McpError::InvalidParams(e.to_string()))?;

        let flow = DeviceFlow::new(self.shared.client_id.clone(), self.shared.scopes.clone());
        match flow.poll_once(&parsed.device_code).await {
            Ok(DevicePollOutcome::Granted(mut tokens)) => {
                // Verify the granted identity matches the configured account
                // before persisting — otherwise a human consenting as the wrong
                // Google account would silently rebind this server to that mailbox.
                let granted_email =
                    fetch_gmail_profile_email(&tokens.access_token)
                        .await
                        .map_err(|e| {
                            McpError::ToolError(anyhow::anyhow!(
                                "auth completed but identity verification failed: {e}"
                            ))
                        })?;

                if !granted_account_matches(&self.shared.account, &granted_email) {
                    // Do NOT write to store and do NOT invalidate — leave the
                    // previous identity intact until a correct re-auth happens.
                    return Ok(serde_json::json!({
                        "status": "wrong_account",
                        "expected_account": self.shared.account,
                        "granted_email": granted_email,
                        "next_action": "The user signed in as a different Google account than this server is configured for. Call gmail_begin_auth again and ensure the user consents with the expected account."
                    }));
                }

                tokens.account_email = granted_email.clone();
                self.shared
                    .store
                    .put(&self.shared.account, &tokens)
                    .await
                    .map_err(|e| McpError::ToolError(anyhow::anyhow!(e)))?;
                self.shared.token_manager.invalidate().await;
                Ok(serde_json::json!({
                    "status": "ok",
                    "account": self.shared.account,
                    "granted_email": granted_email,
                    "next_action": "Re-authentication complete. Retry the Gmail tool call that previously failed."
                }))
            }
            Ok(DevicePollOutcome::Pending { slow_down_extra }) => Ok(serde_json::json!({
                "status": "pending",
                "retry_after_seconds": 5 + slow_down_extra.unwrap_or(0),
                "next_action": "Human has not consented yet. Wait retry_after_seconds, then call gmail_complete_auth again with the same device_code."
            })),
            Err(AuthError::UserDenied) => Ok(serde_json::json!({
                "status": "denied",
                "next_action": "User denied consent. If they intended to allow it, call gmail_begin_auth to retry."
            })),
            Err(AuthError::Timeout) => Ok(serde_json::json!({
                "status": "expired",
                "next_action": "Device code expired. Call gmail_begin_auth to start a fresh flow."
            })),
            Err(e) => Err(McpError::ToolError(anyhow::anyhow!(e))),
        }
    }
}

/// Fetch the authenticated user's email via Gmail's profile endpoint.
/// Works with any Gmail scope, so we don't need `email`/`openid` in the
/// granted scope set just to verify identity.
async fn fetch_gmail_profile_email(access_token: &str) -> Result<String, AuthError> {
    #[derive(serde::Deserialize)]
    struct Profile {
        #[serde(rename = "emailAddress")]
        email_address: String,
    }
    let profile: Profile = reqwest::Client::new()
        .get("https://gmail.googleapis.com/gmail/v1/users/me/profile")
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(profile.email_address)
}

/// Server admins may use a non-email label for `--account`. Only treat it as
/// an identity claim worth enforcing when it has the shape of an email.
fn account_looks_like_email(s: &str) -> bool {
    s.contains('@')
}

/// Decide whether the identity the human consented as (`granted_email`) is
/// acceptable for the server's configured `account`. A non-email account label
/// is not an identity claim, so it always matches; an email-shaped account must
/// match the granted email case-insensitively. Returning `false` triggers the
/// `wrong_account` guard that refuses to persist the token.
fn granted_account_matches(account: &str, granted_email: &str) -> bool {
    !account_looks_like_email(account) || granted_email.eq_ignore_ascii_case(account)
}

#[cfg(test)]
mod tests {
    use super::granted_account_matches;

    #[test]
    fn non_email_account_label_always_matches() {
        // A label like "work" is not an identity claim — any granted email is fine.
        assert!(granted_account_matches("work", "anyone@gmail.com"));
        assert!(granted_account_matches("default", "someone.else@example.com"));
    }

    #[test]
    fn matching_email_account_matches_case_insensitively() {
        assert!(granted_account_matches("Alice@Gmail.com", "alice@gmail.com"));
        assert!(granted_account_matches("alice@gmail.com", "ALICE@GMAIL.COM"));
    }

    #[test]
    fn mismatched_email_account_is_rejected() {
        assert!(!granted_account_matches("alice@gmail.com", "bob@gmail.com"));
    }
}
