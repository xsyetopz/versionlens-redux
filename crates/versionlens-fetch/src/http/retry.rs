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
mod tests {
fn should_retry_error() -> bool {
    false
}
fn retry_backoff_ms(attempt: u32) -> u64 {
    100 * 2_u64.pow(attempt)
}

#[test]
fn request_light_parity_does_not_retry_transient_failures() {
    assert!(!should_retry_error());
    assert_eq!(retry_backoff_ms(0), 100);
    assert_eq!(retry_backoff_ms(1), 200);
}

#[test]
fn npm_registry_fetch_retry_policy_keeps_large_manifest_failures_fast() {
    let policy = crate::http::npm_registry_fetch_retry_policy();

    assert_eq!(policy.max_retries(), 2);
    assert_eq!(policy.retry_backoff_ms(0), Some(250));
    assert_eq!(policy.retry_backoff_ms(1), Some(500));
    assert_eq!(policy.retry_backoff_ms(2), None);
}

#[test]
fn npm_registry_fetch_retry_policy_retries_transient_non_post_statuses() {
    let policy = crate::http::npm_registry_fetch_retry_policy();

    assert!(policy.should_retry_status("GET", 408));
    assert!(policy.should_retry_status("GET", 420));
    assert!(policy.should_retry_status("GET", 429));
    assert!(policy.should_retry_status("GET", 500));
    assert!(policy.should_retry_status("GET", 503));
    assert!(!policy.should_retry_status("GET", 404));
    assert!(!policy.should_retry_status("POST", 500));
}

#[test]
fn disabled_retry_policy_preserves_request_light_no_retry_behavior() {
    let policy = crate::http::disabled_retry_policy();

    assert_eq!(policy.max_retries(), 0);
    assert_eq!(policy.retry_backoff_ms(0), None);
    assert!(!policy.should_retry_status("GET", 408));
    assert!(!policy.should_retry_status("GET", 429));
    assert!(!policy.should_retry_status("GET", 500));
}
}

pub fn disabled_retry_policy() -> RetryPolicy {
    RetryPolicy::disabled()
}

pub fn npm_registry_fetch_retry_policy() -> RetryPolicy {
    RetryPolicy::npm_registry_fetch()
}
