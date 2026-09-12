use std::time::{Duration, Instant};

use super::*;

#[test]
fn unanswered_workspace_edits_finish_once_at_the_deadline_or_cancellation() -> Result<()> {
    for cancelled in [false, true] {
        let (server, client) = Connection::memory();
        let mut queue = WorkQueue::new();
        let now = Instant::now();
        let original = RequestId::from(1);
        let application = RequestId::from("versionlens/apply/1".to_owned());
        queue.applications.insert(
            application.clone(),
            PendingApplication {
                original: original.clone(),
                deadline: now + Duration::from_secs(30),
            },
        );
        queue.expire_applications(&server, now)?;
        assert!(client.receiver.try_recv().is_err());
        if cancelled {
            queue.cancel(&server, &original)?;
        } else {
            queue.expire_applications(&server, now + Duration::from_secs(30))?;
        }
        let Message::Notification(cancellation) =
            client.receiver.recv_timeout(Duration::from_secs(1))?
        else {
            anyhow::bail!("expected cancellation of the editor request");
        };
        assert_eq!(cancellation.method, "$/cancelRequest");
        assert_eq!(cancellation.params, json!({"id":application}));
        let Message::Response(response) = client.receiver.recv_timeout(Duration::from_secs(1))?
        else {
            anyhow::bail!("expected completion of the original command");
        };
        assert_eq!(response.id, original);
        assert_eq!(
            response.response_result.unwrap_err().code,
            if cancelled {
                ErrorCode::RequestCanceled
            } else {
                ErrorCode::RequestFailed
            } as i32
        );
        queue.applied(
            &server,
            Response::new_ok(application, json!({"applied":true})),
        )?;
        queue.expire_applications(&server, now + Duration::from_secs(60))?;
        assert!(queue.applications.is_empty());
        assert!(client.receiver.try_recv().is_err());
    }
    Ok(())
}
