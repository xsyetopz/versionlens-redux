use std::time::Duration;

use super::RequestDeadline;

#[test]
fn request_deadline_reports_expiry() {
    let deadline = RequestDeadline::after(Some(Duration::ZERO));

    assert!(deadline.ensure_remaining().is_err());
}
