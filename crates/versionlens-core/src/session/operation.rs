use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::contract::AuthorizationRequestPayload;

#[derive(Debug)]
pub(crate) struct OperationContext {
    authorization_requests: Mutex<Vec<AuthorizationRequestPayload>>,
    deadline: Option<Instant>,
}

impl OperationContext {
    pub(crate) fn with_timeout(timeout: Duration) -> Self {
        Self {
            authorization_requests: crate::mutex(vec![]),
            deadline: Instant::now().checked_add(timeout),
        }
    }

    pub(crate) fn is_expired(&self) -> bool {
        self.remaining_duration()
            .is_some_and(|remaining| remaining.is_zero())
    }

    pub(crate) fn remaining_duration(&self) -> Option<Duration> {
        self.deadline
            .map(|deadline| deadline.saturating_duration_since(Instant::now()))
    }

    pub(crate) fn record_authorization_request(&self, auth_url: String, request_url: String) {
        let mut requests = self
            .authorization_requests
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned));
        if requests
            .iter()
            .any(|request| request.auth_url == auth_url && request.request_url == request_url)
        {
            return;
        }
        requests.push(AuthorizationRequestPayload {
            auth_url,
            request_url,
        });
    }

    pub(crate) fn take_authorization_requests(&self) -> Vec<AuthorizationRequestPayload> {
        self.authorization_requests
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
            .drain(..)
            .collect()
    }
}

impl Default for OperationContext {
    fn default() -> Self {
        Self {
            authorization_requests: crate::mutex(vec![]),
            deadline: None,
        }
    }
}

#[cfg(test)]
mod tests;
