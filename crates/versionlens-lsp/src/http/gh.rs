//! The GitHub CLI's token, the last fallback after the `githubToken` setting and the
//! `GITHUB_TOKEN` and `GH_TOKEN` variables, so a signed-in `gh` lifts the anonymous rate limit.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tokio::sync::OnceCell;

/// How long `gh auth token` may run; a stuck keychain prompt must not hold lookups.
const LIMIT: Duration = Duration::from_secs(2);

/// Read once per process, on the first GitHub request that has no other token.
static TOKEN: OnceCell<Option<String>> = OnceCell::const_new();

/// The token `gh auth token --hostname github.com` prints, or `None` when `gh` is missing,
/// signed out, or slower than [`LIMIT`].
pub(crate) async fn token() -> Option<String> {
    TOKEN
        .get_or_init(|| async {
            tokio::task::spawn_blocking(|| {
                run("gh", &["auth", "token", "--hostname", "github.com"])
            })
            .await
            .ok()
            .flatten()
        })
        .await
        .clone()
}

/// Runs `program` and returns its trimmed standard output when it exits successfully within
/// [`LIMIT`], killing it otherwise.
pub(crate) fn run(program: &str, args: &[&str]) -> Option<String> {
    run_in(Path::new("."), program, args)
}

/// [`run`] in the directory `dir`, for a command whose answer depends on where it runs, such as
/// `dotnet nuget list source`, which reads the `nuget.config` files above it.
pub(crate) fn run_in(dir: &Path, program: &str, args: &[&str]) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().ok()? {
            break status;
        }
        if started.elapsed() > LIMIT {
            // Reaped so no zombie is left; the run gives no token whether or not the kill lands.
            return child.kill().and_then(|()| child.wait()).ok().and(None);
        }
        thread::sleep(Duration::from_millis(20));
    };
    let mut out = String::new();
    child.stdout.take()?.read_to_string(&mut out).ok()?;
    let out = out.trim();
    (status.success() && !out.is_empty()).then(|| out.to_owned())
}

#[cfg(test)]
mod tests;
