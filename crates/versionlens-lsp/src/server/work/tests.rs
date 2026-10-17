use std::sync::LazyLock;
use std::time::{Duration, Instant};

use super::*;

fn test_runtime() -> tokio::runtime::Handle {
    static RUNTIME: LazyLock<tokio::runtime::Runtime> =
        LazyLock::new(|| tokio::runtime::Runtime::new().expect("test runtime"));
    RUNTIME.handle().clone()
}

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

fn cached_test_document() -> (VersionLensLspState, DocumentWork, WorkQueue) {
    let mut state = VersionLensLspState::standard();
    let work = open_test_document(&mut state);
    let mut queue = WorkQueue::new(test_runtime());
    cache_test_document(&mut queue, &work);
    (state, work, queue)
}

#[test]
fn completed_lenses_are_reused_without_submitting_session_work() -> Result<()> {
    let (server, client) = Connection::memory();
    let (state, work, mut queue) = cached_test_document();

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
    let (mut state, _work, mut queue) = cached_test_document();

    state.invalidate_workspace();
    queue.invalidate(&state, &server)?;

    assert!(queue.completed.is_empty());
    Ok(())
}

#[test]
fn unanswered_workspace_edits_finish_once_at_the_deadline_or_cancellation() -> Result<()> {
    for cancelled in [false, true] {
        let (server, client) = Connection::memory();
        let mut queue = WorkQueue::new(test_runtime());
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

/// A lens request must not wait on the persistent cache. With `generation.json`
/// gone, reading the cache epoch takes `cache.lock`, which another process can hold.
#[test]
fn lens_requests_do_not_wait_for_the_persistent_cache_lock() -> Result<()> {
    let directory = versionlens_test_support::temporary_directory("lens-cache-lock")?;
    let mut state = VersionLensLspState::standard();
    state.use_persistent_cache(&directory)?;
    let uri = "file:///workspace/Cargo.toml";
    state.open_document(crate::state::VersionLensTextDocument {
        uri: uri.to_owned(),
        language_id: "toml".to_owned(),
        text: "[package]\nname = \"example\"\n".to_owned(),
        workspace_root: None,
    });
    let _ = std::fs::remove_file(directory.join("generation.json"));
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(directory.join("cache.lock"))?;
    lock.lock()?;
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(2));
        drop(lock);
    });
    let (server, client) = Connection::memory();
    let mut queue = WorkQueue::new(test_runtime());

    let started = Instant::now();
    queue.lenses(&state, &server, RequestId::from(1), uri)?;
    let elapsed = started.elapsed();
    release
        .join()
        .map_err(|_| anyhow::anyhow!("lock holder panicked"))?;
    assert!(
        elapsed < Duration::from_millis(500),
        "lens request blocked the protocol thread for {elapsed:?}"
    );

    let completion = queue.receiver.recv_timeout(Duration::from_secs(5))?;
    queue.complete(&state, &server, completion)?;
    let Message::Response(response) = client.receiver.recv_timeout(Duration::from_secs(1))? else {
        anyhow::bail!("expected the code lens response first");
    };
    assert_eq!(response.id, RequestId::from(1));
    assert!(response.response_result.is_ok_and(|value| value.is_array()));
    Ok(())
}

/// Opening and editing a document run on the protocol thread, so they must
/// not read the persistent cache, whose lock another process can hold.
#[test]
fn opening_and_changing_documents_do_not_wait_for_the_persistent_cache_lock() -> Result<()> {
    let directory = versionlens_test_support::temporary_directory("open-cache-lock")?;
    let mut state = VersionLensLspState::standard();
    state.use_persistent_cache(&directory)?;
    let _ = std::fs::remove_file(directory.join("generation.json"));
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(directory.join("cache.lock"))?;
    lock.lock()?;
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(2));
        drop(lock);
    });
    let uri = "file:///workspace/Cargo.toml";

    let started = Instant::now();
    state.open_document(crate::state::VersionLensTextDocument {
        uri: uri.to_owned(),
        language_id: "toml".to_owned(),
        text: "[dependencies]\nserde = \"1\"\n".to_owned(),
        workspace_root: None,
    });
    state.change_document(uri, "[dependencies]\nserde = \"1.0\"\n".to_owned());
    let elapsed = started.elapsed();
    release
        .join()
        .map_err(|_| anyhow::anyhow!("lock holder panicked"))?;
    assert!(
        elapsed < Duration::from_millis(500),
        "document notifications blocked the protocol thread for {elapsed:?}"
    );
    Ok(())
}
