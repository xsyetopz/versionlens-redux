use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Priority {
    Interactive,
    Background,
}

#[derive(Default)]
struct Queue {
    interactive: VecDeque<Job>,
    background: VecDeque<Job>,
    interactive_streak: usize,
}

impl Queue {
    fn full(&self, priority: Priority) -> bool {
        self.interactive.len() + self.background.len() >= 64
            || matches!(priority, Priority::Background) && self.background.len() >= 48
    }

    fn next(&mut self, background_allowed: bool) -> Option<Job> {
        if background_allowed
            && !self.background.is_empty()
            && (self.interactive.is_empty() || self.interactive_streak >= 3)
        {
            self.interactive_streak = 0;
            return self.background.pop_front();
        }
        let job = self.interactive.pop_front();
        if job.is_some() {
            self.interactive_streak = self.interactive_streak.saturating_add(1);
        }
        job
    }
}

#[derive(Default)]
struct ResolutionPool {
    queue: Mutex<Queue>,
    ready: Condvar,
    space: Condvar,
}

impl ResolutionPool {
    fn start(workers: usize, interactive_workers: usize) -> Arc<Self> {
        let pool = Arc::new(Self::default());
        for index in 0..workers {
            let pool = Arc::clone(&pool);
            let background_allowed = index >= interactive_workers;
            thread::spawn(move || {
                loop {
                    let job = {
                        let mut queue = pool.queue.lock().unwrap_or_else(crate::recover_poison);
                        loop {
                            if let Some(job) = queue.next(background_allowed) {
                                pool.space.notify_all();
                                break job;
                            }
                            queue = pool.ready.wait(queue).unwrap_or_else(crate::recover_poison);
                        }
                    };
                    job();
                }
            });
        }
        pool
    }
}

pub(crate) fn schedule(priority: Priority, job: impl FnOnce() + Send + 'static) {
    static POOL: OnceLock<Arc<ResolutionPool>> = OnceLock::new();
    let pool = POOL.get_or_init(|| ResolutionPool::start(8, 0));
    let mut queue = pool.queue.lock().unwrap_or_else(crate::recover_poison);
    while queue.full(priority) {
        queue = pool.space.wait(queue).unwrap_or_else(crate::recover_poison);
    }
    let target = match priority {
        Priority::Interactive => &mut queue.interactive,
        Priority::Background => &mut queue.background,
    };
    target.push_back(Box::new(job));
    drop(queue);
    pool.ready.notify_one();
}

pub(crate) fn schedule_coordinator(
    priority: Priority,
    job: impl FnOnce() + Send + 'static,
) -> Result<(), &'static str> {
    static POOL: OnceLock<Arc<ResolutionPool>> = OnceLock::new();
    let pool = POOL.get_or_init(|| ResolutionPool::start(2, 1));
    let mut queue = pool.queue.lock().unwrap_or_else(crate::recover_poison);
    if queue.full(priority) {
        return Err("resolution queue is full");
    }
    match priority {
        Priority::Interactive => &mut queue.interactive,
        Priority::Background => &mut queue.background,
    }
    .push_back(Box::new(job));
    drop(queue);
    pool.ready.notify_all();
    Ok(())
}

#[cfg(test)]
mod tests;
