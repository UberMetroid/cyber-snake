//! Rate limiting for WebSocket connections.

use tokio::time::Instant;

/// Maximum messages per second per client (rate limiting)
const MAX_MESSAGES_PER_SEC: usize = 100;

/// Token bucket for rate limiting (async-friendly)
pub struct RateLimiter {
    tokens: usize,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            tokens: MAX_MESSAGES_PER_SEC,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        // Refill tokens: 1 token per 10ms
        let new_tokens = elapsed.as_millis() as usize / 10;
        if new_tokens > 0 {
            self.tokens = (self.tokens + new_tokens).min(MAX_MESSAGES_PER_SEC);
            self.last_refill = now;
        }

        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_burst() {
        let mut limiter = RateLimiter::new();
        for _ in 0..MAX_MESSAGES_PER_SEC {
            assert!(limiter.try_consume());
        }
        assert!(!limiter.try_consume());
    }

    #[test]
    fn test_rate_limiter_refills() {
        let mut limiter = RateLimiter::new();
        for _ in 0..MAX_MESSAGES_PER_SEC {
            limiter.try_consume();
        }
        assert!(!limiter.try_consume());

        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(limiter.try_consume());
    }
}
