use tokio::sync::Notify;

use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) struct JobTracker {
    min_jobs: usize,
    running_jobs: AtomicUsize,
    notify: Notify,
}

impl JobTracker {
    pub fn new(min_jobs: usize) -> Self {
        Self {
            min_jobs,
            running_jobs: AtomicUsize::new(0),
            notify: Notify::new(),
        }
    }

    pub fn n_jobs_running(&self) -> usize {
        self.running_jobs.load(Ordering::SeqCst)
    }

    pub fn notified(&self) -> tokio::sync::futures::Notified {
        self.notify.notified()
    }

    pub fn job_execution_inserted(&self) {
        self.notify.notify_one()
    }
}
