use std::time::{Duration, Instant};

use super::*;

fn open_test_document(state: &mut VersionLensLspState) -> DocumentWork {
    let uri = "file:///workspace/Cargo.toml";
    state.open_document(crate::state::VersionLensTextDocument {
        uri: uri.to_owned(),
        language_id: "toml".to_owned(),
        text: "[dependencies]\nserde = \"1\"\n".to_owned(),
        workspace_root: None,
    });
    state.document_work(uri).unwrap()
}

fn cache_test_document(queue: &mut WorkQueue, work: &DocumentWork) {
    queue.completed.insert(
        work.uri.clone(),
        CompletedDocument {
            generation: work.generation,
            resolved: ResolvedDocument {
                code_lenses: Vec::new(),
                diagnostics: Vec::new(),
            },
        },
    );
}

fn cached_test_document() -> Result<(VersionLensLspState, DocumentWork, WorkQueue)> {
    let mut state = VersionLensLspState::standard();
    let work = open_test_document(&mut state);
    let mut queue = WorkQueue::new()?;
    cache_test_document(&mut queue, &work);
    Ok((state, work, queue))
}

#[test]
fn completed_lenses_are_reused_without_submitting_session_work() -> Result<()> {
    let (server, client) = Connection::memory();
    let (state, work, mut queue) = cached_test_document()?;

    queue.lenses(&state, &server, RequestId::from(7), work.uri.as_str())?;

    assert!(queue.pending.is_empty());
    let Message::Response(response) = client.receiver.recv_timeout(Duration::from_secs(1))? else {
        anyhow::bail!("expected cached code-lens response");
    };
    assert_eq!(response.response_result.unwrap(), json!([]));
    let Message::Notification(diagnostics) =
        client.receiver.recv_timeout(Duration::from_secs(1))?
    else {
        anyhow::bail!("expected cached diagnostics");
    };
    assert_eq!(diagnostics.method, "textDocument/publishDiagnostics");
    Ok(())
}

#[test]
fn generation_changes_invalidate_completed_lenses() -> Result<()> {
    let (server, _client) = Connection::memory();
    let (mut state, _work, mut queue) = cached_test_document()?;

    state.invalidate_workspace();
    queue.invalidate(&state, &server)?;

    assert!(queue.completed.is_empty());
    Ok(())
}

#[test]
fn unanswered_workspace_edits_finish_once_at_the_deadline_or_cancellation() -> Result<()> {
    for cancelled in [false, true] {
        let (server, client) = Connection::memory();
        let mut queue = WorkQueue::new()?;
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
