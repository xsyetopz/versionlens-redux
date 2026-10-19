//! Asks the client for a credential with `versionlens/authorization` when a
//! registry refuses a request. The question is sent from the resolution task
//! and the answer is routed back by the message loop, so no thread waits and
//! other documents keep resolving.
//!
//! Each host is asked once per server session. Resolutions that need the same
//! host while the question is open wait for the same answer, and later ones
//! reuse it, whether it was a credential or none. A client that does not know
//! the method answers with an error, which counts as none.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError, Weak};
use std::time::{Duration, SystemTime};

use crossbeam_channel::Sender;
use lsp_server::{Message, Request, RequestId, Response};
use lsp_types::request::Request as _;
use tokio::runtime::Handle;
use tokio::sync::watch;
use versionlens_core::{AuthorizationFuture, AuthorizationSource, Credential};

use super::rate_limit::RateLimitNotices;
use crate::protocol::{AuthorizationCredential, AuthorizationParams, AuthorizationRequest};

const REQUEST_PREFIX: &str = "versionlens/authorization/";

/// Nothing until the client answers, then its credential or none.
type Answer = Option<Option<Credential>>;

/// How long the client has to answer before the question counts as
/// unanswered. A late answer is still used.
pub(super) const ANSWER_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Default)]
struct Asked {
    next: u64,
    /// The answer, or the open question, of each host.
    hosts: HashMap<String, watch::Receiver<Answer>>,
    /// Open questions by request ID, with the host they ask about.
    open: HashMap<RequestId, (String, watch::Sender<Answer>)>,
    /// Questions given up on, which a late answer can still settle.
    expired: HashMap<RequestId, String>,
}

pub(super) struct AuthorizationBroker {
    me: Weak<Self>,
    /// Dropped by [`Self::close`], so the broker does not keep the
    /// connection open after the server stops.
    sender: Mutex<Option<Sender<Message>>>,
    runtime: Handle,
    timeout: Duration,
    asked: Mutex<Asked>,
    notices: RateLimitNotices,
}

/// A credential that arrived after nobody was waiting for it.
pub(super) struct LateCredential {
    pub(super) host: String,
    pub(super) credential: Credential,
}

impl AuthorizationBroker {
    pub(super) fn new(sender: Sender<Message>, runtime: Handle, timeout: Duration) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            me: me.clone(),
            sender: Mutex::new(Some(sender)),
            runtime,
            timeout,
            asked: Mutex::default(),
            notices: RateLimitNotices::default(),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Asked> {
        self.asked.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Answers every open question with no credential and stops sending.
    pub(super) fn close(&self) {
        self.sender
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        let mut asked = self.lock();
        for (_, (_, answer)) in asked.open.drain() {
            let _ = answer.send(Some(None));
        }
        asked.expired.clear();
    }

    /// Gives up on a question the client has not answered. Waiters get no
    /// credential; an answer that comes later is still used.
    fn expire(&self, id: &RequestId) {
        let mut asked = self.lock();
        if let Some((host, answer)) = asked.open.remove(id) {
            let _ = answer.send(Some(None));
            asked.expired.insert(id.clone(), host);
        }
    }

    /// Applies the client's answer to the question it belongs to. Returns the
    /// credential, so the caller can give it to every session and check the
    /// refused dependencies again.
    pub(super) fn answer(&self, response: Response) -> Option<LateCredential> {
        let credential = response
            .response_result
            .ok()
            .and_then(|result| {
                serde_json::from_value::<Option<AuthorizationCredential>>(result).ok()
            })
            .flatten()
            .filter(|credential| !credential.scheme.is_empty() && !credential.token.is_empty())
            .map(|credential| Credential::new(credential.scheme, credential.token));
        let mut asked = self.lock();
        if let Some((host, answer)) = asked.open.remove(&response.id) {
            // Waiting resolutions take the answer themselves. Others may have
            // given up at their own deadline, so every credential is also
            // returned, once per host, to check the refused dependencies again.
            let _ = answer.send(Some(credential.clone()));
            return credential.map(|credential| LateCredential { host, credential });
        }
        let host = asked.expired.remove(&response.id)?;
        let credential = credential?;
        let (answer, receiver) = watch::channel(Some(Some(credential.clone())));
        drop(answer);
        asked.hosts.insert(host.clone(), receiver);
        drop(asked);
        Some(LateCredential { host, credential })
    }

    /// Sends the question about `host` and records it as open.
    fn ask(&self, asked: &mut Asked, host: &str, status: u16) -> watch::Receiver<Answer> {
        let (answer, receiver) = watch::channel(None);
        let id = RequestId::from(format!("{REQUEST_PREFIX}{}", asked.next));
        asked.next += 1;
        let request = Message::Request(Request {
            id: id.clone(),
            method: AuthorizationRequest::METHOD.to_owned(),
            params: serde_json::to_value(AuthorizationParams {
                host: host.to_owned(),
                status,
            })
            .unwrap_or_default(),
        });
        let sent = self
            .sender
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .is_some_and(|sender| sender.send(request).is_ok());
        if sent {
            asked.open.insert(id.clone(), (host.to_owned(), answer));
            let (broker, timeout) = (self.me.clone(), self.timeout);
            self.runtime.spawn(async move {
                tokio::time::sleep(timeout).await;
                if let Some(broker) = broker.upgrade() {
                    broker.expire(&id);
                }
            });
        } else {
            let _ = answer.send(Some(None));
        }
        asked.hosts.insert(host.to_owned(), receiver.clone());
        receiver
    }
}

impl AuthorizationSource for AuthorizationBroker {
    fn authorize(&self, host: &str, status: u16) -> AuthorizationFuture {
        let mut asked = self.lock();
        let mut receiver = match asked.hosts.get(host) {
            Some(receiver) => receiver.clone(),
            None => self.ask(&mut asked, host, status),
        };
        drop(asked);
        Box::pin(async move {
            let answer = receiver
                .wait_for(Option::is_some)
                .await
                .ok()
                .map(|answer| answer.clone());
            answer.flatten().flatten()
        })
    }

    fn rate_limited(&self, host: &str, reset: SystemTime) {
        let sender = self
            .sender
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(sender) = sender {
            self.notices.report(&sender, host, reset);
        }
    }
}

/// Whether a response answers an authorization question.
pub(super) fn is_answer(id: &RequestId) -> bool {
    id.to_string().trim_matches('"').starts_with(REQUEST_PREFIX)
}
