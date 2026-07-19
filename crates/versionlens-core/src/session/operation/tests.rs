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
