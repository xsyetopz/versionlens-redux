#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    max_retries: u32,
    factor: u64,
    min_timeout_ms: u64,
    max_timeout_ms: u64,
}

impl RetryPolicy {
    pub fn disabled() -> Self {
        Self {
            max_retries: 0,
            factor: 1,
            min_timeout_ms: 0,
            max_timeout_ms: 0,
        }
    }

    pub fn npm_registry_fetch() -> Self {
        Self {
            max_retries: 2,
            factor: 2,
            min_timeout_ms: 250,
            max_timeout_ms: 1_000,
        }
    }

    pub fn max_retries(self) -> u32 {
        self.max_retries
    }

    pub fn retry_backoff_ms(self, attempt: u32) -> Option<u64> {
        (attempt < self.max_retries).then(|| self.backoff_ms(attempt))
    }

    pub fn should_retry_status(self, method: &str, status: u16) -> bool {
        self.max_retries > 0
            && !method.eq_ignore_ascii_case("POST")
            && matches!(status, 408 | 420 | 429 | 500..=599)
    }

    pub(crate) fn should_retry_error(self, method: &str, error: &reqwest::Error) -> bool {
        if self.max_retries == 0 || method.eq_ignore_ascii_case("POST") {
            return false;
        }
        error
            .status()
            .is_some_and(|status| self.should_retry_status(method, status.as_u16()))
            || error.is_timeout()
            || error.is_connect()
    }

    fn backoff_ms(self, attempt: u32) -> u64 {
        self.min_timeout_ms
            .saturating_mul(self.factor.saturating_pow(attempt))
            .min(self.max_timeout_ms)
    }
}

#[cfg(test)]
mod tests;

pub fn disabled_retry_policy() -> RetryPolicy {
    RetryPolicy::disabled()
}

pub fn npm_registry_fetch_retry_policy() -> RetryPolicy {
    RetryPolicy::npm_registry_fetch()
}
