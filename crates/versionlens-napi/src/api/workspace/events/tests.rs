use std::sync::{Arc, mpsc};
use std::time::Duration;

use super::{EventQueue, MAX_BYTES, MAX_EVENTS};
use versionlens_core::{WorkspaceCheckEvent, WorkspaceCheckResult};

fn event(generation: u64) -> WorkspaceCheckEvent {
    WorkspaceCheckEvent {
        generation,
        result: WorkspaceCheckResult::Settled,
    }
}

#[test]
fn replacing_or_closing_a_full_queue_releases_obsolete_producers() {
    for close in [false, true] {
        let queue = Arc::new(EventQueue::default());
        for _ in 0..MAX_EVENTS {
            queue.push(event(1));
        }
        let producer = Arc::clone(&queue);
        let (started, ready) = mpsc::channel();
        let (finished, completion) = mpsc::channel();
        let thread = std::thread::spawn(move || {
            started.send(()).unwrap();
            producer.push(event(1));
            finished.send(()).unwrap();
        });
        ready.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(completion.recv_timeout(Duration::from_millis(30)).is_err());
        if close {
            queue.close();
        } else {
            queue.reset(2);
        }
        completion.recv_timeout(Duration::from_secs(1)).unwrap();
        thread.join().unwrap();
        assert!(queue.take().is_empty());
        queue.push(event(2));
        assert_eq!(queue.take().len(), usize::from(!close));
    }
}

fn large_event() -> WorkspaceCheckEvent {
    WorkspaceCheckEvent {
        generation: 1,
        result: WorkspaceCheckResult::Document {
            input: Box::new(versionlens_model::DocumentInput::new(
                "file:///work/package.json",
                "json",
                "x".repeat(4 * 1024 * 1024),
                None,
            )),
            result: Err("check failed".to_owned()),
        },
    }
}

#[test]
fn byte_budget_applies_backpressure_until_results_are_consumed() {
    let queue = Arc::new(EventQueue::default());
    for _ in 0..3 {
        queue.push(large_event());
    }
    let producer = Arc::clone(&queue);
    let (finished, completion) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        producer.push(large_event());
        finished.send(()).unwrap();
    });
    assert!(completion.recv_timeout(Duration::from_millis(30)).is_err());
    assert!(queue.state.lock().unwrap().bytes <= MAX_BYTES);
    assert_eq!(queue.take().len(), 3);
    completion.recv_timeout(Duration::from_secs(1)).unwrap();
    thread.join().unwrap();
    let events = queue.take();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].document.as_ref().unwrap().text.len(),
        4 * 1024 * 1024
    );
    assert_eq!(events[0].message.as_deref(), Some("check failed"));
}
