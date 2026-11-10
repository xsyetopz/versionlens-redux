use std::sync::{Arc, OnceLock};

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::session::operation::OperationContext;

const MAX_REGISTRY_REQUESTS: usize = 16;
const MAX_BACKGROUND_REQUESTS: usize = 12;

static REQUESTS: OnceLock<Arc<Semaphore>> = OnceLock::new();
static BACKGROUND_REQUESTS: OnceLock<Arc<Semaphore>> = OnceLock::new();
type SharedSemaphore = Arc<Semaphore>;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Priority {
    Interactive,
    Background,
}

pub(crate) async fn acquire_registry_permits(
    priority: Priority,
    operation: &OperationContext,
) -> Option<(OwnedSemaphorePermit, Option<OwnedSemaphorePermit>)> {
    let background = if matches!(priority, Priority::Background) {
        Some(acquire(background_requests(), operation).await?)
    } else {
        None
    };
    let request = acquire(requests(), operation).await?;
    Some((request, background))
}

async fn acquire(
    semaphore: &'static SharedSemaphore,
    operation: &OperationContext,
) -> Option<OwnedSemaphorePermit> {
    let acquire = Arc::clone(semaphore).acquire_owned();
    match operation.remaining_duration() {
        Some(remaining) => tokio::time::timeout(remaining, acquire).await.ok()?.ok(),
        None => acquire.await.ok(),
    }
}

fn requests() -> &'static SharedSemaphore {
    REQUESTS.get_or_init(|| Arc::new(Semaphore::new(MAX_REGISTRY_REQUESTS)))
}

fn background_requests() -> &'static SharedSemaphore {
    BACKGROUND_REQUESTS.get_or_init(|| Arc::new(Semaphore::new(MAX_BACKGROUND_REQUESTS)))
}

#[cfg(test)]
mod tests {
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
}
