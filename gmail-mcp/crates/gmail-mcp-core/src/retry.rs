use crate::gmail::errors::GmailError;
use std::future::Future;
use std::time::{Duration, Instant};

pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base: Duration,
    pub max_backoff: Duration,
    pub total_cap: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            base: Duration::from_secs(1),
            max_backoff: Duration::from_secs(30),
            total_cap: Duration::from_secs(120),
        }
    }
}

pub async fn with_retry<F, Fut, T>(op: F, policy: &RetryPolicy) -> Result<T, GmailError>
where
    F: Fn(u32) -> Fut,
    Fut: Future<Output = Result<T, GmailError>>,
{
    let mut attempt = 0u32;
    let mut elapsed = Duration::ZERO;

    loop {
        let start = Instant::now();
        match op(attempt).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                let kind = classify(&e);
                if !kind.retryable() || attempt >= policy.max_attempts {
                    return Err(e);
                }

                let back = backoff(attempt, policy);
                elapsed += start.elapsed() + back;

                if elapsed > policy.total_cap {
                    return Err(GmailError::Transport(format!(
                        "retry budget exhausted after {attempt} attempts"
                    )));
                }

                tokio::time::sleep(back).await;
                attempt += 1;
            }
        }
    }
}

fn backoff(attempt: u32, p: &RetryPolicy) -> Duration {
    let exp = p.base * 2u32.pow(attempt);
    let clamped = exp.min(p.max_backoff);
    let jitter = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_millis()
        % 250;
    clamped + Duration::from_millis(jitter as u64)
}

enum RetryKind {
    NoRetry,
    RetryTransient,
    RetryAuth,
}

impl RetryKind {
    fn retryable(&self) -> bool {
        !matches!(self, Self::NoRetry)
    }
}

fn classify(e: &GmailError) -> RetryKind {
    match e {
        GmailError::RateLimited | GmailError::Transport(_) => RetryKind::RetryTransient,
        GmailError::AuthExpired => RetryKind::RetryAuth,
        _ => RetryKind::NoRetry,
    }
}
