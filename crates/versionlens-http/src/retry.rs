use std::io::ErrorKind;

#[cfg(test)]
const INITIAL_BACKOFF_MS: u64 = 100;

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
        if attempt >= self.max_retries {
            return None;
        }

        Some(self.backoff_ms(attempt))
    }

    pub fn should_retry_status(self, method: &str, status: u16) -> bool {
        self.max_retries > 0
            && !method.eq_ignore_ascii_case("POST")
            && matches!(status, 408 | 420 | 429 | 500..=599)
    }

    pub(crate) fn should_retry_error(self, method: &str, error: &ureq::Error) -> bool {
        if self.max_retries == 0 || method.eq_ignore_ascii_case("POST") {
            return false;
        }

        match error {
            ureq::Error::StatusCode(status) => self.should_retry_status(method, *status),
            ureq::Error::Io(error) => matches!(
                error.kind(),
                ErrorKind::ConnectionReset
                    | ErrorKind::ConnectionRefused
                    | ErrorKind::AddrInUse
                    | ErrorKind::TimedOut
            ),
            ureq::Error::Timeout(_) | ureq::Error::ConnectionFailed => true,
            _ => false,
        }
    }

    fn backoff_ms(self, attempt: u32) -> u64 {
        let multiplier = self.factor.saturating_pow(attempt);
        self.min_timeout_ms
            .saturating_mul(multiplier)
            .min(self.max_timeout_ms)
    }
}

#[cfg(test)]
pub(crate) fn should_retry_error(_error: &ureq::Error) -> bool {
    false
}

#[cfg(test)]
pub(crate) fn retry_backoff_ms(attempt: u32) -> u64 {
    INITIAL_BACKOFF_MS * 2_u64.pow(attempt)
}

#[cfg(test)]
mod tests;
