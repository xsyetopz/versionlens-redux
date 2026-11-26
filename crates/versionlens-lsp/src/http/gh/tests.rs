#![cfg(unix)]

use std::time::Instant;

use super::*;

#[test]
fn returns_trimmed_output_of_successful_run() {
    assert_eq!(
        run("sh", &["-c", "printf ' gho_x\\n'"]).as_deref(),
        Some("gho_x")
    );
}

#[test]
fn returns_none_for_failure_silence_and_missing_program() {
    assert_eq!(run("sh", &["-c", "echo gho_x; exit 1"]), None);
    assert_eq!(run("sh", &["-c", "true"]), None);
    assert_eq!(run("versionlens-no-such-program", &[]), None);
}

#[test]
fn kills_run_past_limit() {
    let started = Instant::now();
    assert_eq!(run("sh", &["-c", "sleep 10; echo late"]), None);
    assert!(started.elapsed() < LIMIT + Duration::from_secs(1));
}
