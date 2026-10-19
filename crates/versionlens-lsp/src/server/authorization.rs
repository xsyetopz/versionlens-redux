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
use std::sync::{Arc, Mutex, PoisonError};

use crossbeam_channel::Sender;
use lsp_server::{Message, Request, RequestId, Response};
use lsp_types::request::Request as _;
use tokio::sync::watch;
use versionlens_core::{AuthorizationFuture, AuthorizationSource, Credential};

use crate::protocol::{AuthorizationCredential, AuthorizationParams, AuthorizationRequest};

const REQUEST_PREFIX: &str = "versionlens/authorization/";

/// Nothing until the client answers, then its credential or none.
type Answer = Option<Option<Credential>>;

#[derive(Default)]
struct Asked {
    next: u64,
    /// The answer, or the open question, of each host.
    hosts: HashMap<String, watch::Receiver<Answer>>,
    /// Open questions by request ID.
    open: HashMap<RequestId, watch::Sender<Answer>>,
}

pub(super) struct AuthorizationBroker {
    sender: Sender<Message>,
    asked: Mutex<Asked>,
}

impl AuthorizationBroker {
    pub(super) fn new(sender: Sender<Message>) -> Arc<Self> {
        Arc::new(Self {
            sender,
            asked: Mutex::default(),
        })
    }

    /// Applies the client's answer to the question it belongs to.
    pub(super) fn answer(&self, response: Response) {
        let sender = self
            .asked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .open
            .remove(&response.id);
        let Some(sender) = sender else {
            return;
        };
        let credential = response
            .response_result
            .ok()
            .and_then(|result| {
                serde_json::from_value::<Option<AuthorizationCredential>>(result).ok()
            })
            .flatten()
            .filter(|credential| !credential.scheme.is_empty() && !credential.token.is_empty())
            .map(|credential| Credential::new(credential.scheme, credential.token));
        let _ = sender.send(Some(credential));
    }
}

impl AuthorizationBroker {
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
        if self.sender.send(request).is_ok() {
            asked.open.insert(id, answer);
        } else {
            let _ = answer.send(Some(None));
        }
        asked.hosts.insert(host.to_owned(), receiver.clone());
        receiver
    }
}

impl AuthorizationSource for AuthorizationBroker {
    fn authorize(&self, host: &str, status: u16) -> AuthorizationFuture {
        let mut asked = self.asked.lock().unwrap_or_else(PoisonError::into_inner);
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
}

/// Whether a response answers an authorization question.
pub(super) fn is_answer(id: &RequestId) -> bool {
    id.to_string().trim_matches('"').starts_with(REQUEST_PREFIX)
}
