use super::{MAX_BACKGROUND_REQUESTS, MAX_REGISTRY_REQUESTS, background_requests, requests};

#[test]
fn registry_admission_reserves_four_interactive_slots() {
    assert_eq!(requests().available_permits(), MAX_REGISTRY_REQUESTS);
    assert_eq!(
        background_requests().available_permits(),
        MAX_BACKGROUND_REQUESTS
    );
    assert_eq!(MAX_REGISTRY_REQUESTS - MAX_BACKGROUND_REQUESTS, 4);
}
