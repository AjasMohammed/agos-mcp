use std::time::{Duration, Instant};

pub struct RateLimiter {
    capacity: u32,
    refill_per_sec: u32,
    tokens: tokio::sync::Mutex<BucketState>,
}

struct BucketState {
    tokens: f64,
    last: Instant,
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Would block too long: {0:?}")]
    WouldBlockTooLong(Duration),
}

impl RateLimiter {
    pub fn new(rate: u32) -> Self {
        Self {
            capacity: rate,
            refill_per_sec: rate,
            tokens: tokio::sync::Mutex::new(BucketState {
                tokens: rate as f64,
                last: Instant::now(),
            }),
        }
    }

    pub async fn acquire(&self, cost: u32) -> Result<(), RateLimitError> {
        loop {
            let wait = {
                let mut state = self.tokens.lock().await;
                let now = Instant::now();
                let elapsed = now.duration_since(state.last).as_secs_f64();
                state.tokens =
                    (state.tokens + elapsed * self.refill_per_sec as f64).min(self.capacity as f64);
                state.last = now;

                if state.tokens >= cost as f64 {
                    state.tokens -= cost as f64;
                    return Ok(());
                }

                let deficit = cost as f64 - state.tokens;
                Duration::from_secs_f64(deficit / self.refill_per_sec as f64)
            };

            if wait > Duration::from_secs(30) {
                return Err(RateLimitError::WouldBlockTooLong(wait));
            }

            tokio::time::sleep(wait).await;
        }
    }
}
