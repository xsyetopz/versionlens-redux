//! Tells the user once when a registry host has stopped answering because its
//! rate limit is exhausted. The fetch layer holds the host until the reset,
//! so the same reset reaches here once per stopped request; only the first
//! becomes a `window/showMessage`.

use std::collections::HashSet;
use std::sync::{Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use crossbeam_channel::Sender;
use lsp_server::{Message, Notification};
use lsp_types::notification::{Notification as _, ShowMessage};
use lsp_types::{MessageType, ShowMessageParams};

/// The hosts and resets already reported, as host and epoch seconds.
#[derive(Default)]
pub(super) struct RateLimitNotices {
    reported: Mutex<HashSet<(String, u64)>>,
}

impl RateLimitNotices {
    /// Sends the warning for `host` and `reset` unless it was sent before.
    pub(super) fn report(&self, sender: &Sender<Message>, host: &str, reset: SystemTime) {
        let seconds = reset
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs());
        let first = self
            .reported
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert((host.to_owned(), seconds));
        if !first {
            return;
        }
        let params = ShowMessageParams {
            typ: MessageType::WARNING,
            message: message(host, seconds),
        };
        let _ = sender.send(Message::Notification(Notification::new(
            ShowMessage::METHOD.to_owned(),
            params,
        )));
    }
}

fn message(host: &str, reset_seconds: u64) -> String {
    let minutes = reset_seconds / 60 % (24 * 60);
    let reset = format!("{:02}:{:02} UTC", minutes / 60, minutes % 60);
    let name = host.split(':').next().unwrap_or(host);
    if name == "github.com" || name.ends_with(".github.com") {
        return format!(
            "GitHub rate limit reached. VersionLens pauses GitHub requests until {reset}. \
             Set GITHUB_TOKEN or run `gh auth login` to raise the limit."
        );
    }
    format!("{host} rate limit reached. VersionLens pauses requests to it until {reset}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_hosts_say_how_to_authenticate() {
        let text = message("api.github.com", 14 * 3600 + 5 * 60);
        assert!(text.contains("14:05 UTC"), "{text}");
        assert!(text.contains("GITHUB_TOKEN") && text.contains("gh auth login"));
    }

    #[test]
    fn other_hosts_get_the_reset_time_only() {
        let text = message("registry.example:8080", 3600);
        assert!(text.contains("registry.example:8080") && text.contains("01:00 UTC"));
        assert!(!text.contains("GITHUB_TOKEN"));
    }

    #[test]
    fn one_warning_per_host_and_reset() {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let notices = RateLimitNotices::default();
        let reset = UNIX_EPOCH + std::time::Duration::from_secs(1000);
        notices.report(&sender, "a", reset);
        notices.report(&sender, "a", reset);
        notices.report(&sender, "b", reset);
        notices.report(&sender, "a", reset + std::time::Duration::from_secs(60));
        assert_eq!(receiver.try_iter().count(), 3);
    }
}
