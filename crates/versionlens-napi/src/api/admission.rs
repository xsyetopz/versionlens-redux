use std::sync::Mutex;

use super::TaskInput;
use super::weights::document_weight;

const MAX_TASKS: usize = 64;
const MAX_BYTES: usize = 64 * 1024 * 1024;
static GATE: Gate = Gate(Mutex::new((0, 0)));

struct Gate(Mutex<(usize, usize)>);

pub(super) struct Admission {
    gate: &'static Gate,
    bytes: usize,
}

impl Gate {
    fn acquire(&'static self, bytes: usize) -> Result<Admission, &'static str> {
        let mut usage = self.0.lock().unwrap_or_else(crate::recover_poison);
        if usage.0 >= MAX_TASKS || bytes > MAX_BYTES.saturating_sub(usage.1) {
            return Err("Native checking capacity reached; retry after pending checks finish");
        }
        usage.0 += 1;
        usage.1 += bytes;
        drop(usage);
        Ok(Admission { gate: self, bytes })
    }
}

impl Admission {
    pub(super) fn acquire(input: &TaskInput) -> Result<Self, &'static str> {
        let bytes = match input {
            TaskInput::Resolve(input, _) => document_weight(input),
            TaskInput::Apply(input) => document_weight(&input.document)
                .saturating_add(input.command.as_ref().map_or(0, String::capacity))
                .saturating_add(input.dependency_name.as_ref().map_or(0, String::capacity))
                .saturating_add(input.selected_version.as_ref().map_or(0, String::capacity)),
        };
        GATE.acquire(bytes.saturating_add(size_of::<TaskInput>()))
    }
}

impl Drop for Admission {
    fn drop(&mut self) {
        let mut usage = self.gate.0.lock().unwrap_or_else(crate::recover_poison);
        usage.0 -= 1;
        usage.1 -= self.bytes;
    }
}

#[cfg(test)]
mod tests;
