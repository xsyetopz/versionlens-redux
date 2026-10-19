use std::process::{Command, Output, Stdio};
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

const DOTNET_SOURCE_ARGS: &[&str] = &["nuget", "list", "source", "--format", "short"];
const DOTNET_SOURCE_TIMEOUT: Duration = crate::duration_from_millis(750);
const DOTNET_SOURCE_POLL_INTERVAL: Duration = crate::duration_from_millis(25);

pub(super) fn dotnet_source_listing() -> Option<String> {
    let started = Instant::now();
    let mut child = Command::new("dotnet")
        .args(DOTNET_SOURCE_ARGS)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    loop {
        if child.try_wait().ok()?.is_some() {
            return source_listing_from_output(child.wait_with_output().ok()?);
        }

        if started.elapsed() >= DOTNET_SOURCE_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }

        sleep(DOTNET_SOURCE_POLL_INTERVAL);
    }
}

fn source_listing_from_output(output: Output) -> Option<String> {
    if !output.status.success() {
        return None;
    }

    crate::string_from_utf8(output.stdout)
        .ok()
        .filter(|listing| !listing.trim().is_empty())
}
