use crate::auth::TokenManager;
use crate::gmail::errors::{GmailError, map_gmail_error};
use crate::gmail::types::*;
use crate::ratelimit::RateLimiter;
use crate::retry::{RetryPolicy, with_retry};
use reqwest::Method;
use std::sync::Arc;
use std::time::Duration;

/// Default token-bucket: 250 quota units/second, safely below Gmail's limit.
const DEFAULT_RATE: u32 = 250;

pub struct Client {
    pub(crate) http: reqwest::Client,
    pub(crate) tokens: Arc<TokenManager>,
    base: &'static str,
    limiter: Arc<RateLimiter>,
    retry_policy: RetryPolicy,
}

impl Client {
    pub async fn new(tokens: Arc<TokenManager>) -> Self {
        Self::with_rate(tokens, DEFAULT_RATE)
    }

    pub fn with_rate(tokens: Arc<TokenManager>, rate: u32) -> Self {
        let http = reqwest::Client::builder()
            .user_agent(concat!("gmail-mcp/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .https_only(true)
            .build()
            .expect("reqwest builder");
        Self {
            http,
            tokens,
            base: "https://gmail.googleapis.com/gmail/v1",
            limiter: Arc::new(RateLimiter::new(rate)),
            retry_policy: RetryPolicy::default(),
        }
    }

    /// Serialize query/body upfront so the retry closure can be `'static + Clone`.
    async fn request<Q, B, R>(
        &self,
        method: Method,
        path: &str,
        query: Option<&Q>,
        body: Option<&B>,
        cost: u32,
    ) -> Result<R, GmailError>
    where
        Q: serde::Serialize + ?Sized,
        B: serde::Serialize + ?Sized,
        R: for<'de> serde::Deserialize<'de>,
    {
        self.limiter
            .acquire(cost)
            .await
            .map_err(|_| GmailError::RateLimited)?;

        // Serialize eagerly so the retry closure owns `'static` data.
        let query_pairs: Option<Vec<(String, String)>> = query
            .map(|q| {
                serde_json::to_value(q)
                    .map_err(|e| GmailError::Other(e.to_string()))
                    .map(|v| {
                        v.as_object()
                            .map(|m| {
                                m.iter()
                                    .filter_map(|(k, v)| {
                                        let s = match v {
                                            serde_json::Value::String(s) => Some(s.clone()),
                                            serde_json::Value::Number(n) => Some(n.to_string()),
                                            serde_json::Value::Bool(b) => Some(b.to_string()),
                                            serde_json::Value::Null => None,
                                            _ => None,
                                        };
                                        s.map(|sv| (k.clone(), sv))
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default()
                    })
            })
            .transpose()?;

        let body_bytes: Option<Vec<u8>> = body
            .map(|b| serde_json::to_vec(b))
            .transpose()
            .map_err(|e| GmailError::Other(e.to_string()))?;

        let token = self.tokens.access_token().await?;
        let url = format!("{}/{}", self.base, path.trim_start_matches('/'));
        let http = self.http.clone();

        with_retry(
            |_attempt| {
                let http = http.clone();
                let method = method.clone();
                let url = url.clone();
                let token = token.clone();
                let query_pairs = query_pairs.clone();
                let body_bytes = body_bytes.clone();
                async move {
                    let mut req = http.request(method, &url).bearer_auth(&token);
                    if let Some(ref pairs) = query_pairs {
                        req = req.query(pairs);
                    }
                    if let Some(ref bytes) = body_bytes {
                        req = req
                            .header("Content-Type", "application/json")
                            .body(bytes.clone());
                    }
                    let resp = req.send().await?;
                    let status = resp.status();
                    if status.is_success() {
                        return Ok(resp.json::<R>().await?);
                    }
                    Err(map_gmail_error(status, resp).await)
                }
            },
            &self.retry_policy,
        )
        .await
    }

    async fn request_empty<B>(
        &self,
        method: Method,
        path: &str,
        cost: u32,
        body: Option<&B>,
    ) -> Result<(), GmailError>
    where
        B: serde::Serialize + ?Sized,
    {
        self.limiter
            .acquire(cost)
            .await
            .map_err(|_| GmailError::RateLimited)?;

        let body_bytes: Option<Vec<u8>> = body
            .map(|b| serde_json::to_vec(b))
            .transpose()
            .map_err(|e| GmailError::Other(e.to_string()))?;

        let token = self.tokens.access_token().await?;
        let url = format!("{}/{}", self.base, path.trim_start_matches('/'));
        let http = self.http.clone();

        with_retry(
            |_attempt| {
                let http = http.clone();
                let method = method.clone();
                let url = url.clone();
                let token = token.clone();
                let body_bytes = body_bytes.clone();
                async move {
                    let mut req = http.request(method, &url).bearer_auth(&token);
                    if let Some(ref bytes) = body_bytes {
                        req = req
                            .header("Content-Type", "application/json")
                            .body(bytes.clone());
                    }
                    let resp = req.send().await?;
                    let status = resp.status();
                    if status.is_success() {
                        return Ok(());
                    }
                    Err(map_gmail_error(status, resp).await)
                }
            },
            &self.retry_policy,
        )
        .await
    }

    // ── Read-only ─────────────────────────────────────────────────────────────

    pub async fn messages_list(
        &self,
        q: &MessagesListQuery,
    ) -> Result<MessagesListResp, GmailError> {
        self.request(Method::GET, "users/me/messages", Some(q), None::<&()>, 5)
            .await
    }

    pub async fn messages_get(&self, id: &str, fmt: MessageFormat) -> Result<Message, GmailError> {
        let q = [("format", fmt.as_str())];
        self.request(
            Method::GET,
            &format!("users/me/messages/{id}"),
            Some(&q),
            None::<&()>,
            5,
        )
        .await
    }

    pub async fn threads_get(&self, id: &str) -> Result<Thread, GmailError> {
        self.request(
            Method::GET,
            &format!("users/me/threads/{id}"),
            None::<&()>,
            None::<&()>,
            10,
        )
        .await
    }

    pub async fn labels_list(&self) -> Result<LabelsListResp, GmailError> {
        self.request(Method::GET, "users/me/labels", None::<&()>, None::<&()>, 1)
            .await
    }

    pub async fn labels_get(&self, id: &str) -> Result<Label, GmailError> {
        self.request(
            Method::GET,
            &format!("users/me/labels/{id}"),
            None::<&()>,
            None::<&()>,
            1,
        )
        .await
    }

    pub async fn filters_list(&self) -> Result<FiltersListResp, GmailError> {
        self.request(
            Method::GET,
            "users/me/settings/filters",
            None::<&()>,
            None::<&()>,
            1,
        )
        .await
    }

    pub async fn filters_get(&self, id: &str) -> Result<Filter, GmailError> {
        self.request(
            Method::GET,
            &format!("users/me/settings/filters/{id}"),
            None::<&()>,
            None::<&()>,
            1,
        )
        .await
    }

    pub async fn drafts_list(&self) -> Result<DraftsListResp, GmailError> {
        self.request(Method::GET, "users/me/drafts", None::<&()>, None::<&()>, 5)
            .await
    }

    pub async fn profile_get(&self) -> Result<Profile, GmailError> {
        self.request(Method::GET, "users/me/profile", None::<&()>, None::<&()>, 1)
            .await
    }

    pub async fn attachment_get(
        &self,
        msg_id: &str,
        att_id: &str,
    ) -> Result<Attachment, GmailError> {
        self.request(
            Method::GET,
            &format!("users/me/messages/{msg_id}/attachments/{att_id}"),
            None::<&()>,
            None::<&()>,
            5,
        )
        .await
    }

    // ── Write ─────────────────────────────────────────────────────────────────

    pub async fn messages_send(
        &self,
        raw: &str,
        thread_id: Option<&str>,
    ) -> Result<MessageRef, GmailError> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            raw: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            thread_id: Option<&'a str>,
        }
        self.request(
            Method::POST,
            "users/me/messages/send",
            None::<&()>,
            Some(&Body { raw, thread_id }),
            100,
        )
        .await
    }

    pub async fn drafts_create(&self, raw: &str) -> Result<DraftRef, GmailError> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            message: Msg<'a>,
        }
        #[derive(serde::Serialize)]
        struct Msg<'a> {
            raw: &'a str,
        }
        self.request(
            Method::POST,
            "users/me/drafts",
            None::<&()>,
            Some(&Body {
                message: Msg { raw },
            }),
            10,
        )
        .await
    }

    pub async fn drafts_update(&self, id: &str, raw: &str) -> Result<DraftRef, GmailError> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            message: Msg<'a>,
        }
        #[derive(serde::Serialize)]
        struct Msg<'a> {
            raw: &'a str,
        }
        self.request(
            Method::PUT,
            &format!("users/me/drafts/{id}"),
            None::<&()>,
            Some(&Body {
                message: Msg { raw },
            }),
            10,
        )
        .await
    }

    pub async fn drafts_send(&self, id: &str) -> Result<MessageRef, GmailError> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            id: &'a str,
        }
        self.request(
            Method::POST,
            "users/me/drafts/send",
            None::<&()>,
            Some(&Body { id }),
            100,
        )
        .await
    }

    pub async fn drafts_delete(&self, id: &str) -> Result<(), GmailError> {
        self.request_empty(
            Method::DELETE,
            &format!("users/me/drafts/{id}"),
            5,
            None::<&()>,
        )
        .await
    }

    pub async fn messages_modify(
        &self,
        id: &str,
        add: &[String],
        remove: &[String],
    ) -> Result<Message, GmailError> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            #[serde(rename = "addLabelIds")]
            add: &'a [String],
            #[serde(rename = "removeLabelIds")]
            remove: &'a [String],
        }
        self.request(
            Method::POST,
            &format!("users/me/messages/{id}/modify"),
            None::<&()>,
            Some(&Body { add, remove }),
            5,
        )
        .await
    }

    pub async fn messages_trash(&self, id: &str) -> Result<Message, GmailError> {
        self.request(
            Method::POST,
            &format!("users/me/messages/{id}/trash"),
            None::<&()>,
            None::<&()>,
            5,
        )
        .await
    }

    pub async fn messages_untrash(&self, id: &str) -> Result<Message, GmailError> {
        self.request(
            Method::POST,
            &format!("users/me/messages/{id}/untrash"),
            None::<&()>,
            None::<&()>,
            5,
        )
        .await
    }

    // ── Labels CRUD ───────────────────────────────────────────────────────────

    pub async fn labels_create(&self, name: &str) -> Result<Label, GmailError> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            name: &'a str,
        }
        self.request(
            Method::POST,
            "users/me/labels",
            None::<&()>,
            Some(&Body { name }),
            5,
        )
        .await
    }

    pub async fn labels_update(&self, id: &str, name: Option<&str>) -> Result<Label, GmailError> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<&'a str>,
        }
        self.request(
            Method::PATCH,
            &format!("users/me/labels/{id}"),
            None::<&()>,
            Some(&Body { name }),
            5,
        )
        .await
    }

    pub async fn labels_delete(&self, id: &str) -> Result<(), GmailError> {
        self.request_empty(
            Method::DELETE,
            &format!("users/me/labels/{id}"),
            5,
            None::<&()>,
        )
        .await
    }

    // ── Filters CRUD ──────────────────────────────────────────────────────────

    pub async fn filters_create(&self, body: &serde_json::Value) -> Result<Filter, GmailError> {
        self.request(
            Method::POST,
            "users/me/settings/filters",
            None::<&()>,
            Some(body),
            5,
        )
        .await
    }

    pub async fn filters_delete(&self, id: &str) -> Result<(), GmailError> {
        self.request_empty(
            Method::DELETE,
            &format!("users/me/settings/filters/{id}"),
            5,
            None::<&()>,
        )
        .await
    }

    // ── Batch operations ──────────────────────────────────────────────────────

    pub async fn messages_batch_modify(&self, body: &serde_json::Value) -> Result<(), GmailError> {
        self.request_empty(
            Method::POST,
            "users/me/messages/batchModify",
            50,
            Some(body),
        )
        .await
    }

    pub async fn messages_batch_delete(&self, ids: &[String]) -> Result<(), GmailError> {
        #[derive(serde::Serialize)]
        struct Body<'a> {
            ids: &'a [String],
        }
        self.request_empty(
            Method::POST,
            "users/me/messages/batchDelete",
            50,
            Some(&Body { ids }),
        )
        .await
    }
}
