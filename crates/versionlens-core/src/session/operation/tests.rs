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
    let operation = super::OperationContext::with_timeout(std::time::Duration::from_secs(1))
        .with_generation(&epoch, None);
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
