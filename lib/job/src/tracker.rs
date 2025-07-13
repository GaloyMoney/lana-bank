use tokio::sync::Notify;

use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) struct JobTracker {
    running_jobs: AtomicUsize,
    notify: Notify,
}

impl JobTracker {
    pub fn new() -> Self {
        Self {
            running_jobs: AtomicUsize::new(0),
            notify: Notify::new(),
        }
    }
}
