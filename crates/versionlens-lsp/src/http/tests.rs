use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use super::*;
use crate::registry;

/// Answers each request with the page its query names: `name=20` and `name=alpine` get their
/// filtered pages, any other the first page. Returns the base URL.
fn hub() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut request = Vec::new();
            let mut buf = [0; 1024];
            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                match stream.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => request.extend_from_slice(&buf[..n]),
                }
            }
            let request = String::from_utf8_lossy(&request);
            let body = if request.contains("name=20 ") {
                r#"{"results":[{"name":"20-alpine"},{"name":"20.19.6-alpine"},{"name":"26"}]}"#
            } else if request.contains("name=alpine ") {
                r#"{"results":[{"name":"26-alpine"},{"name":"20-alpine"}]}"#
            } else {
                r#"{"results":[{"name":"26"},{"name":"26-bookworm"}]}"#
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            drop(stream.write_all(response.as_bytes()));
        }
    });
    format!("http://127.0.0.1:{port}")
}

fn docker_at(base: &str) -> &'static Registry {
    let docker = registry::get("docker").unwrap();
    let url = format!("{base}/{{docker}}");
    let filtered = format!("{base}/{{docker}}?name={{filter}}");
    Box::leak(Box::new(Registry {
        id: docker.id,
        url: Box::leak(url.into_boxed_str()),
        parse: registry::Parse::Json {
            at: "results",
            field: "name",
            latest: "",
        },
        bare: docker.bare,
        osv: docker.osv,
        osv_name: docker.osv_name,
        accept: docker.accept,
        yanked: docker.yanked,
        deprecated: docker.deprecated,
        filtered: Box::leak(filtered.into_boxed_str()),
        fallback: docker.fallback,
        paged: docker.paged,
    }))
}

fn list(registry: &'static Registry, filters: &[&str]) -> Vec<String> {
    let http = Http::new(Settings::default()).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime
        .block_on(http.versions(
            registry,
            "node",
            None,
            &filters.iter().map(|f| (*f).to_owned()).collect::<Vec<_>>(),
        ))
        .unwrap()
        .list
}

#[test]
fn joins_the_filtered_pages_without_repeats() {
    let docker = docker_at(&hub());
    assert_eq!(
        list(docker, &["20", "alpine"]),
        [
            "26",
            "26-bookworm",
            "20-alpine",
            "20.19.6-alpine",
            "26-alpine"
        ]
    );
    assert_eq!(list(docker, &[]), ["26", "26-bookworm"]);
}
