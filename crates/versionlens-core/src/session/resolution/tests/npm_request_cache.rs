use super::{DocumentInput, RegistryResponseInput, session_without_vulnerabilities};
use std::env;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::PathBuf;
use std::process::id;
use std::sync::Barrier;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::thread::current;
use std::thread::scope;
use std::thread::sleep;
use std::thread::spawn;
use versionlens_parsers::Ecosystem::Npm;

#[test]
fn registry_context_resolution_reuses_cached_response_body_by_url() {
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
    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "registry-context-resolution-reuses-cached-response-body-by-url.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };

    let first = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
        }],
    );
    let second = session.resolve_document_with_responses(input, &[]);

    assert_eq!(first.edits[0].new_text, "1.1.0");
    assert_eq!(second.edits[0].new_text, "1.1.0");

    remove_dir_all(root).unwrap();
}

fn barrier(count: usize) -> Barrier {
    <Barrier>::new(count)
}

#[test]
fn concurrent_registry_resolution_deduplicates_inflight_request_body_fetches() {
    use std::io::{Read, Write};
    use std::sync::atomic::Ordering::SeqCst;

    let listener = crate::tcp_listener_bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let registry_url = format!("http://{}/", listener.local_addr().unwrap());
    let request_count = crate::arc(<AtomicUsize>::default());
    let stop = crate::arc(<AtomicBool>::default());
    let server_request_count = crate::clone_arc(&request_count);
    let server_stop = crate::clone_arc(&stop);
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
    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "concurrent-registry-resolution-deduplicates-inflight-request-body-fetches.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let barrier = crate::arc(barrier(2));

    scope(|scope| {
        let session_ref = &session;
        let first_barrier = crate::clone_arc(&barrier);
        let first_input = input.clone();
        let first = scope.spawn(move || {
            first_barrier.wait();
            session_ref.resolve_document_with_responses(first_input, &[])
        });
        let session_ref = &session;
        let second_barrier = crate::clone_arc(&barrier);
        let second = scope.spawn(move || {
            second_barrier.wait();
            session_ref.resolve_document_with_responses(input, &[])
        });

        assert_eq!(first.join().unwrap().edits[0].new_text, "1.1.0");
        assert_eq!(second.join().unwrap().edits[0].new_text, "1.1.0");
    });

    stop.store(true, SeqCst);
    server.join().unwrap();
    assert_eq!(request_count.load(SeqCst), 1);

    remove_dir_all(root).unwrap();
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/npm_request_cache")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}
