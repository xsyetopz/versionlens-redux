//! A way for a server to supply a credential after a registry refused a
//! request, such as by asking the editor.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, PoisonError, RwLock};

use versionlens_fetch::Credential;

use super::VersionLensSession;

/// The credential for a host, or `None` when there is none to offer.
pub type AuthorizationFuture = Pin<Box<dyn Future<Output = Option<Credential>> + Send>>;

/// Asked for a credential when a host answers 401 or 403 and no earlier
/// source helps. Implementations must not block the thread that polls them,
/// and should ask at most once per host.
pub trait AuthorizationSource: Send + Sync {
    fn authorize(&self, host: &str, status: u16) -> AuthorizationFuture;
}

/// The source shared by a session and its clones.
#[derive(Clone, Default)]
pub(crate) struct AuthorizationHook(Arc<RwLock<Option<Arc<dyn AuthorizationSource>>>>);

impl fmt::Debug for AuthorizationHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AuthorizationHook")
    }
}

impl AuthorizationHook {
    pub(crate) fn get(&self) -> Option<Arc<dyn AuthorizationSource>> {
        self.0
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl VersionLensSession {
    /// Sets the source asked for a credential after a 401 or 403 that no
    /// client, environment or `gh` credential resolves. The credential it
    /// returns is kept for the rest of the session and the request is retried.
    pub fn set_authorization_source(&self, source: Arc<dyn AuthorizationSource>) {
        *self
            .request_state
            .authorization
            .0
            .write()
            .unwrap_or_else(PoisonError::into_inner) = Some(source);
    }
}
