use super::{Gate, MAX_BYTES, MAX_TASKS};
use std::sync::Mutex;

#[test]
fn pending_native_work_obeys_task_and_byte_limits_and_releases_capacity() {
    static GATE: Gate = Gate(Mutex::new((0, 0)));
    let mut permits = (0..MAX_TASKS)
        .map(|_| GATE.acquire(1).unwrap())
        .collect::<Vec<_>>();
    assert!(GATE.acquire(1).is_err());
    permits.pop();
    assert!(GATE.acquire(1).is_ok());
    drop(permits);
    let full = GATE.acquire(MAX_BYTES).unwrap();
    assert!(GATE.acquire(1).is_err());
    drop(full);
    assert!(GATE.acquire(MAX_BYTES + 1).is_err());
    assert!(GATE.acquire(MAX_BYTES).is_ok());
}
