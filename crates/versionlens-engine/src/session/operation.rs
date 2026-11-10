use std::sync::PoisonError;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::contract::AuthorizationRequestPayload;

#[derive(Debug, Clone)]
pub(crate) struct OperationContext {
    authorization_requests: Arc<Mutex<Vec<AuthorizationRequestPayload>>>,
    deadline: Option<Instant>,
    pub(crate) attempted_at_ms: u64,
    pub(crate) persistent_epoch: Option<u64>,
    storage: Option<Arc<versionlens_fetch::PersistentCache>>,
    cancellation: Option<Arc<std::sync::atomic::AtomicBool>>,
    generation: Option<(Arc<std::sync::atomic::AtomicU64>, u64)>,
}

impl OperationContext {
    pub(crate) fn with_timeout(timeout: Duration) -> Self {
        Self {
            authorization_requests: Arc::new(Mutex::new(vec![])),
            deadline: Instant::now().checked_add(timeout),
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
        storage: Option<Arc<versionlens_fetch::PersistentCache>>,
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
            .unwrap_or_else(PoisonError::into_inner);
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
            .unwrap_or_else(PoisonError::into_inner)
            .drain(..)
            .collect()
    }
}

impl Default for OperationContext {
    fn default() -> Self {
        Self {
            authorization_requests: Arc::new(Mutex::new(vec![])),
            deadline: None,
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
mod tests {
    use std::thread::scope;
    use std::time::Duration;

    use super::OperationContext;

    #[test]
    fn concurrent_operations_keep_authorization_requests_isolated() {
        let first = OperationContext::default();
        let second = OperationContext::default();

        scope(|scope| {
            scope.spawn(|| {
                for _ in 0..32 {
                    first.record_authorization_request(
                        "https://first.example.test".to_owned(),
                        "https://first.example.test/package".to_owned(),
                    );
                }
            });
            scope.spawn(|| {
                for _ in 0..32 {
                    second.record_authorization_request(
                        "https://second.example.test".to_owned(),
                        "https://second.example.test/package".to_owned(),
                    );
                }
            });
        });

        let first_requests = first.take_authorization_requests();
        let second_requests = second.take_authorization_requests();
        assert_eq!(first_requests.len(), 1);
        assert_eq!(second_requests.len(), 1);
        assert_eq!(first_requests[0].auth_url, "https://first.example.test");
        assert_eq!(second_requests[0].auth_url, "https://second.example.test");
    }

    #[test]
    fn zero_timeout_expires_operation_immediately() {
        let operation = OperationContext::with_timeout(Duration::ZERO);

        assert!(operation.is_expired());
        assert_eq!(operation.remaining_duration(), Some(Duration::ZERO));
    }

    #[test]
    fn operations_without_deadlines_expose_no_remaining_duration() {
        let operation = OperationContext::default();

        assert!(!operation.is_expired());
        assert_eq!(operation.remaining_duration(), None);
    }

    #[test]
    fn cloned_operation_preserves_generation_and_authorization_collection() {
        let epoch = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let operation =
            OperationContext::with_timeout(Duration::from_secs(1)).with_generation(&epoch, None);
        let execution = operation.clone();
        execution.record_authorization_request(
            "https://registry.test".to_owned(),
            "https://registry.test/package".to_owned(),
        );
        assert_eq!(operation.take_authorization_requests().len(), 1);
        epoch.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        assert!(!execution.is_current());
        assert!(execution.is_expired());
    }

    #[test]
    fn cancelling_one_task_preserves_other_operations() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        let cancelled = Arc::new(AtomicBool::new(false));
        let operation = OperationContext::default().with_cancellation(Some(Arc::clone(&cancelled)));
        let independent = OperationContext::default();
        cancelled.store(true, Ordering::Release);
        assert!(!operation.can_publish());
        assert_eq!(operation.remaining_duration(), Some(Duration::ZERO));
        assert!(independent.can_publish());
    }

    #[test]
    fn queued_operations_keep_the_submission_generation() {
        let epoch = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(2));
        let operation = OperationContext::default().with_generation(&epoch, Some(1));
        assert!(!operation.is_current());
    }
}
