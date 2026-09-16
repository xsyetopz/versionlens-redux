use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::thread::current;
use std::thread::sleep;
use std::thread::spawn;

#[tokio::test]
async fn registry_context_resolution_reuses_cached_response_body_by_url() {
    let root = temp_dir().join(format!(
        "versionlens-request-cache-{}-{}",
        id(),
        current().name().unwrap_or("test")
    ));
    create_dir_all(&root).unwrap();
    write(
        root.join(".npmrc"),
        "registry=https://registry.example.test/\n",
    )
    .unwrap();

    let session = session_without_vulnerabilities();
    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture("registry-context-resolution-reuses-cached-response-body-by-url.txt"),
        Some(root.to_string_lossy().into_owned()),
    );

    let first = session
        .resolve_document_with_responses(
            input.clone(),
            &[crate::support::tests::npm_latest_response(
                "left-pad", "1.1.0",
            )],
        )
        .await;
    let second = session.resolve_document_with_responses(input, &[]).await;

    assert_eq!(first.edits[0].new_text, "1.1.0");
    assert_eq!(second.edits[0].new_text, "1.1.0");

    remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn concurrent_registry_resolution_deduplicates_inflight_request_body_fetches() {
    use std::io::{Read, Write};
    use std::sync::atomic::Ordering::SeqCst;

    let listener = crate::support::tests::tcp_listener_bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let registry_url = format!("http://{}/", listener.local_addr().unwrap());
    let request_count = Arc::new(<AtomicUsize>::default());
    let stop = Arc::new(<AtomicBool>::default());
    let server_request_count = crate::support::tests::clone_arc(&request_count);
    let server_stop = crate::support::tests::clone_arc(&stop);
    let server = spawn(move || {
        while !server_stop.load(SeqCst) {
            let Ok((mut stream, _)) = listener.accept() else {
                sleep(crate::duration_from_millis(5));
                continue;
            };
            server_request_count.fetch_add(1, SeqCst);
            let mut buffer = [0_u8; 1024];
            let _ = stream.read(&mut buffer);
            sleep(crate::duration_from_millis(75));
            let body = r#"{"dist-tags":{"latest":"1.1.0"}}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });

    let root = temp_dir().join(format!(
        "versionlens-inflight-request-cache-{}-{}",
        id(),
        current().name().unwrap_or("test")
    ));
    create_dir_all(&root).unwrap();
    write(root.join(".npmrc"), format!("registry={registry_url}\n")).unwrap();

    let session = session_without_vulnerabilities();
    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture(
            "concurrent-registry-resolution-deduplicates-inflight-request-body-fetches.txt",
        ),
        Some(root.to_string_lossy().into_owned()),
    );
    let barrier = Arc::new(tokio::sync::Barrier::new(3));
    let first_session = session.clone();
    let first_barrier = crate::support::tests::clone_arc(&barrier);
    let first_input = input.clone();
    let first = tokio::spawn(async move {
        first_barrier.wait().await;
        first_session
            .resolve_document_with_responses(first_input, &[])
            .await
    });
    let second_session = session.clone();
    let second_barrier = crate::support::tests::clone_arc(&barrier);
    let second = tokio::spawn(async move {
        second_barrier.wait().await;
        second_session
            .resolve_document_with_responses(input, &[])
            .await
    });
    barrier.wait().await;
    let (first, second) = tokio::join!(first, second);
    assert_eq!(first.unwrap().edits[0].new_text, "1.1.0");
    assert_eq!(second.unwrap().edits[0].new_text, "1.1.0");

    stop.store(true, SeqCst);
    server.join().unwrap();
    assert_eq!(request_count.load(SeqCst), 1);

    remove_dir_all(root).unwrap();
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture(
        "tests/fixtures/session/resolution/tests/npm_request_cache",
        name,
    )
}

#[tokio::test]
async fn background_resolution_runs_twelve_registry_requests_and_preserves_order() {
    use std::io::{Read, Write};
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::{Condvar, Mutex};
    use std::time::Duration;

    const UNIQUE_REQUESTS: usize = 12;
    let listener = crate::support::tests::tcp_listener_bind("127.0.0.1:0").unwrap();
    let registry_url = format!("http://{}/", listener.local_addr().unwrap());
    let in_flight = Arc::new((Mutex::new(0_usize), Condvar::new()));
    let request_count = Arc::new(AtomicUsize::new(0));
    let server_in_flight = crate::support::tests::clone_arc(&in_flight);
    let server_request_count = crate::support::tests::clone_arc(&request_count);
    let server = spawn(move || {
        let mut workers = Vec::new();
        for _ in 0..UNIQUE_REQUESTS {
            let (mut stream, _) = listener.accept().unwrap();
            let worker_in_flight = crate::support::tests::clone_arc(&server_in_flight);
            let worker_request_count = crate::support::tests::clone_arc(&server_request_count);
            workers.push(spawn(move || {
                let mut buffer = [0_u8; 1024];
                assert!(stream.read(&mut buffer).unwrap() > 0);
                worker_request_count.fetch_add(1, SeqCst);
                let (lock, ready) = &*worker_in_flight;
                let mut count = lock.lock().unwrap();
                *count += 1;
                ready.notify_all();
                while *count < UNIQUE_REQUESTS {
                    let (next, timeout) =
                        ready.wait_timeout(count, Duration::from_secs(5)).unwrap();
                    assert!(
                        !timeout.timed_out(),
                        "registry requests did not run concurrently"
                    );
                    count = next;
                }
                drop(count);
                let body = r#"{"dist-tags":{"latest":"2.0.0"}}"#;
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            }));
        }
        for worker in workers {
            worker.join().unwrap();
        }
    });

    let dependencies = (0..UNIQUE_REQUESTS)
        .map(|index| format!(r#""pkg{index}":"1.0.0""#))
        .chain(std::iter::once(r#""alias":"npm:pkg0@1.0.0""#.to_owned()))
        .collect::<Vec<_>>()
        .join(",");
    let mut config = crate::support::tests::session_config(crate::default(), false);
    config.providers.registry_urls = vec![RegistryUrlConfig {
        ecosystem: Npm,
        url: registry_url,
    }];
    let mut session = VersionLensSession::new(config);
    session.storage_state.task_priority = crate::concurrency::Priority::Background;
    let output = session
        .resolve_document(DocumentInput::new(
            "file:///package.json",
            "json",
            format!(r#"{{"dependencies":{{{dependencies}}}}}"#),
            None,
        ))
        .await;
    server.join().unwrap();

    let names = output
        .suggestions
        .iter()
        .map(|suggestion| suggestion.dependency.name.as_str())
        .collect::<Vec<_>>();
    let expected = (0..UNIQUE_REQUESTS)
        .map(|index| format!("pkg{index}"))
        .chain(std::iter::once("pkg0".to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(names, expected);
    assert_eq!(request_count.load(SeqCst), UNIQUE_REQUESTS);
}
