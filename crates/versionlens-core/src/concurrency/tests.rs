use super::{Priority, Queue};

#[test]
fn background_work_receives_a_slot_after_three_interactive_tasks() {
    let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut queue = Queue::default();
    for index in 0..4 {
        let results = std::sync::Arc::clone(&results);
        queue.interactive.push_back(Box::new(move || {
            results.lock().unwrap().push(index);
        }));
    }
    let background = std::sync::Arc::clone(&results);
    queue.background.push_back(Box::new(move || {
        background.lock().unwrap().push(9);
    }));
    while let Some(job) = queue.next(true) {
        job();
    }
    assert_eq!(*results.lock().unwrap(), [0, 1, 2, 9, 3]);
}

#[test]
fn reserved_worker_leaves_background_work_for_the_shared_worker() {
    let mut queue = Queue::default();
    queue.background.push_back(Box::new(|| {}));
    assert!(queue.next(false).is_none());
    assert_eq!(queue.background.len(), 1);
    queue.interactive.push_back(Box::new(|| {}));
    assert!(queue.next(false).is_some());
    assert_eq!(queue.background.len(), 1);
    assert!(queue.next(true).is_some());
    assert!(queue.background.is_empty());
}

#[test]
fn queue_reserves_capacity_for_interactive_requests() {
    let mut queue = Queue::default();
    for _ in 0..48 {
        queue.background.push_back(Box::new(|| {}));
    }
    assert!(queue.full(Priority::Background));
    assert!(!queue.full(Priority::Interactive));
    for _ in 0..16 {
        queue.interactive.push_back(Box::new(|| {}));
    }
    assert!(queue.full(Priority::Interactive));
}
