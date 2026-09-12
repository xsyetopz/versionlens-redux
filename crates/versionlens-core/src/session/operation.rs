use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::contract::AuthorizationRequestPayload;

#[derive(Debug, Clone)]
pub(crate) struct OperationContext {
    authorization_requests: Arc<Mutex<Vec<AuthorizationRequestPayload>>>,
    deadline: Option<Instant>,
    timeout: Option<Duration>,
    pub(crate) attempted_at_ms: u64,
    pub(crate) persistent_epoch: Option<u64>,
    storage: Option<Arc<versionlens_cache::PersistentCache>>,
    cancellation: Option<Arc<std::sync::atomic::AtomicBool>>,
    generation: Option<(Arc<std::sync::atomic::AtomicU64>, u64)>,
}

impl OperationContext {
    pub(crate) fn with_timeout(timeout: Duration) -> Self {
        Self {
            authorization_requests: Arc::new(crate::mutex(vec![])),
            deadline: Instant::now().checked_add(timeout),
            timeout: Some(timeout),
            attempted_at_ms: timestamp_ms(),
            generation: None,
            cancellation: None,
            persistent_epoch: None,
            storage: None,
        }
    }

    pub(crate) fn independent_timeout(&self, timeout: Duration) -> Self {
        Self {
            authorization_requests: Arc::clone(&self.authorization_requests),
            deadline: Instant::now().checked_add(timeout),
            timeout: Some(timeout),
            attempted_at_ms: timestamp_ms(),
            generation: self.generation.clone(),
            cancellation: self.cancellation.clone(),
            persistent_epoch: self.persistent_epoch,
            storage: self.storage.clone(),
        }
    }

    pub(crate) fn with_generation(
        mut self,
        epoch: &Arc<std::sync::atomic::AtomicU64>,
        expected: Option<u64>,
    ) -> Self {
        self.generation = Some((
            Arc::clone(epoch),
            expected.unwrap_or_else(|| epoch.load(std::sync::atomic::Ordering::Acquire)),
        ));
        self
    }

    pub(crate) fn with_storage(
        mut self,
        storage: Option<Arc<versionlens_cache::PersistentCache>>,
        epoch: Option<u64>,
    ) -> Self {
        self.storage = storage;
        self.persistent_epoch = epoch;
        self
    }

    pub(crate) fn with_cancellation(
        mut self,
        cancellation: Option<Arc<std::sync::atomic::AtomicBool>>,
    ) -> Self {
        self.cancellation = cancellation;
        self
    }

    pub(crate) fn can_publish(&self) -> bool {
        self.is_current()
            && self
                .storage
                .as_ref()
                .zip(self.persistent_epoch)
                .is_none_or(|(cache, epoch)| cache.epoch().ok() == Some(epoch))
    }

    pub(crate) fn is_current(&self) -> bool {
        self.cancellation
            .as_ref()
            .is_none_or(|cancelled| !cancelled.load(std::sync::atomic::Ordering::Acquire))
            && self.generation.as_ref().is_none_or(|(epoch, expected)| {
                epoch.load(std::sync::atomic::Ordering::Acquire) == *expected
            })
    }

    pub(crate) fn for_execution(&self) -> Self {
        match self.timeout {
            Some(timeout) => self.independent_timeout(timeout),
            None => self.clone(),
        }
    }

    pub(crate) fn is_expired(&self) -> bool {
        !self.is_current()
            || self
                .remaining_duration()
                .is_some_and(|remaining| remaining.is_zero())
    }

    pub(crate) fn remaining_duration(&self) -> Option<Duration> {
        if !self.is_current() {
            return Some(Duration::ZERO);
        }
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
            authorization_requests: Arc::new(crate::mutex(vec![])),
            deadline: None,
            timeout: None,
            attempted_at_ms: timestamp_ms(),
            generation: None,
            cancellation: None,
            persistent_epoch: None,
            storage: None,
        }
    }
}

pub(super) fn timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().try_into().unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests;
