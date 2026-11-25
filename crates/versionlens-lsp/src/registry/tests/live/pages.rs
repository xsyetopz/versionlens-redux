//! Asking for one package page the way a browser would, through the challenges and passing
//! outages public hosts answer with, and failing only on an answer that stays wrong.

use std::time::Duration;

use reqwest::StatusCode;

/// The agent a page is first asked with, which pages that sniff for a browser want.
pub(super) const BROWSER: &str = "Mozilla/5.0 (versionlens live check)";
const PLAIN: &str = "versionlens live check";

/// How often a page is asked, and the first wait before asking again, which doubles each try up
/// to eight times itself.
#[derive(Clone, Copy)]
pub(super) struct Policy {
    pub(super) tries: u32,
    pub(super) wait: Duration,
}

/// Asks for `url` until it answers with success, and returns why it did not otherwise.
///
/// - Anubis, in front of code.forgejo.org, answers a `Mozilla` agent with a 202 proof-of-work page
///   and lets other agents through: its default `data/botPolicies.yaml` weighs only a
///   `Mozilla|Opera` agent toward a challenge (github.com/TecharoHQ/anubis), so a 202 is asked
///   for again with a plain agent, whose answer stands for the page.
/// - A 5xx, such as the 502 ghcr.io's front end sometimes sends, and a timeout or refused
///   connection say nothing about the page, so it is asked again; one that lasts every try fails.
/// - A Cloudflare challenge (`cf-mitigated`) is asked again the same way.
/// - Any other answer, such as a 404, fails at once.
pub(super) async fn check(
    client: &reqwest::Client,
    url: &str,
    clearance: Option<(String, String)>,
    policy: Policy,
) -> Option<String> {
    let mut last = String::new();
    for attempt in 0..policy.tries {
        if attempt > 0 {
            tokio::time::sleep(policy.wait * (1 << (attempt - 1).min(3))).await;
        }
        let mut request = client
            .get(url)
            .header("accept", "text/html")
            .header("user-agent", BROWSER);
        if let Some((agent, cookie)) = &clearance {
            request = request.header("user-agent", agent).header("cookie", cookie);
        }
        let mut answer = request.send().await;
        if answer
            .as_ref()
            .is_ok_and(|r| r.status() == StatusCode::ACCEPTED)
        {
            answer = client.get(url).header("user-agent", PLAIN).send().await;
        }
        match answer {
            Ok(response) if response.status().is_success() => {
                eprintln!("page {url}: HTTP {}", response.status());
                return None;
            }
            Ok(response) if response.headers().contains_key("cf-mitigated") => {
                last = format!("{url}: Cloudflare challenge on every try");
            }
            Ok(response) if response.status().is_server_error() => {
                last = format!("{url}: HTTP {} on every try", response.status());
            }
            Ok(response) => return Some(format!("{url}: HTTP {}", response.status())),
            Err(e) if e.is_timeout() || e.is_connect() => {
                last = format!("{url}: {} on every try", crate::http::chain(&e));
            }
            Err(e) => return Some(format!("{url}: {}", crate::http::chain(&e))),
        }
    }
    Some(last)
}

/// A local server whose answer to the `n`th request, from 0, with user agent `agent` is the
/// status `answer` gives. Returns its URL and the count of requests it answered.
async fn serve(
    answer: fn(usize, &str) -> u16,
) -> (String, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
    use std::sync::atomic::Ordering;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/page", listener.local_addr().unwrap());
    let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let served = std::sync::Arc::clone(&count);
    tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut head = Vec::new();
            let mut byte = [0; 1];
            while !head.ends_with(b"\r\n\r\n") && stream.read(&mut byte).await.unwrap() == 1 {
                head.push(byte[0]);
            }
            let head = String::from_utf8_lossy(&head).to_ascii_lowercase();
            let agent = head
                .lines()
                .find_map(|l| l.strip_prefix("user-agent: "))
                .unwrap_or_default();
            let status = answer(served.fetch_add(1, Ordering::SeqCst), agent);
            let reply =
                format!("HTTP/1.1 {status} X\r\ncontent-length: 0\r\nconnection: close\r\n\r\n");
            stream.write_all(reply.as_bytes()).await.unwrap();
        }
    });
    (url, count)
}

fn run(answer: fn(usize, &str) -> u16) -> (Option<String>, usize) {
    // Run alone, a test builds the first client, so it installs the provider `Http::new` would.
    drop(rustls::crypto::ring::default_provider().install_default());
    super::block_on(async {
        let (url, count) = serve(answer).await;
        let policy = Policy {
            tries: 4,
            wait: Duration::from_millis(5),
        };
        let failed = check(&reqwest::Client::new(), &url, None, policy).await;
        (failed, count.load(std::sync::atomic::Ordering::SeqCst))
    })
}

#[test]
fn passing_bad_gateway_is_asked_again() {
    // Arrange
    let answer = |n, _: &str| if n < 2 { 502 } else { 200 };

    // Act
    let (failed, asked) = run(answer);

    // Assert
    assert_eq!((failed, asked), (None, 3));
}

#[test]
fn lasting_bad_gateway_fails_after_every_try() {
    // Arrange
    let answer = |_, _: &str| 502;

    // Act
    let (failed, asked) = run(answer);

    // Assert
    assert!(
        failed
            .unwrap()
            .ends_with("HTTP 502 Bad Gateway on every try")
    );
    assert_eq!(asked, 4);
}

#[test]
fn missing_page_fails_at_once() {
    // Arrange
    let answer = |_, _: &str| 404;

    // Act
    let (failed, asked) = run(answer);

    // Assert
    assert!(failed.unwrap().ends_with("HTTP 404 Not Found"));
    assert_eq!(asked, 1);
}

#[test]
fn anubis_challenge_resolves_with_plain_agent() {
    // Arrange: Anubis answers a browser with a 202 challenge and a plain agent with the page.
    let answer = |_, agent: &str| if agent.contains("mozilla") { 202 } else { 200 };

    // Act
    let (failed, asked) = run(answer);

    // Assert
    assert_eq!((failed, asked), (None, 2));
}
