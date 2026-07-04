use super::{DocumentInput, Ecosystem, RegistryResponseInput, session_without_vulnerabilities};

#[test]
fn registry_context_resolution_reuses_cached_response_body_by_url() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-request-cache-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "registry=https://registry.example.test/\n",
    )
    .unwrap();

    let session = session_without_vulnerabilities();
    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };

    let first = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
        }],
    );
    let second = session.resolve_document_with_responses(input, &[]);

    assert_eq!(first.edits[0].new_text, "1.1.0");
    assert_eq!(second.edits[0].new_text, "1.1.0");

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn concurrent_registry_resolution_deduplicates_inflight_request_body_fetches() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{
        Arc, Barrier,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    };
    use std::time::Duration;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let registry_url = format!("http://{}/", listener.local_addr().unwrap());
    let request_count = Arc::new(AtomicUsize::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let server_request_count = Arc::clone(&request_count);
    let server_stop = Arc::clone(&stop);
    let server = std::thread::spawn(move || {
        while !server_stop.load(Ordering::SeqCst) {
            let Ok((mut stream, _)) = listener.accept() else {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            };
            server_request_count.fetch_add(1, Ordering::SeqCst);
            let mut buffer = [0_u8; 1024];
            let _ = stream.read(&mut buffer);
            std::thread::sleep(Duration::from_millis(75));
            let body = r#"{"dist-tags":{"latest":"1.1.0"}}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });

    let root = std::env::temp_dir().join(format!(
        "versionlens-inflight-request-cache-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(".npmrc"), format!("registry={registry_url}\n")).unwrap();

    let session = session_without_vulnerabilities();
    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let barrier = Arc::new(Barrier::new(2));

    std::thread::scope(|scope| {
        let session_ref = &session;
        let first_barrier = Arc::clone(&barrier);
        let first_input = input.clone();
        let first = scope.spawn(move || {
            first_barrier.wait();
            session_ref.resolve_document_with_responses(first_input, &[])
        });
        let session_ref = &session;
        let second_barrier = Arc::clone(&barrier);
        let second = scope.spawn(move || {
            second_barrier.wait();
            session_ref.resolve_document_with_responses(input, &[])
        });

        assert_eq!(first.join().unwrap().edits[0].new_text, "1.1.0");
        assert_eq!(second.join().unwrap().edits[0].new_text, "1.1.0");
    });

    stop.store(true, Ordering::SeqCst);
    server.join().unwrap();
    assert_eq!(request_count.load(Ordering::SeqCst), 1);

    std::fs::remove_dir_all(root).unwrap();
}
