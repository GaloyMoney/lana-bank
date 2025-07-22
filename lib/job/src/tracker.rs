use tokio::sync::Notify;

use std::sync::atomic::{AtomicUsize, Ordering};

pub(crate) struct JobTracker {
    min_jobs: usize,
    max_jobs: usize,
    running_jobs: AtomicUsize,
    notify: Notify,
}

impl JobTracker {
    pub fn new(min_jobs: usize, max_jobs: usize) -> Self {
        Self {
            min_jobs,
            max_jobs,
            running_jobs: AtomicUsize::new(0),
            notify: Notify::new(),
        }
    }

    pub fn next_batch_size(&self) -> Option<usize> {
        let n_running = self.running_jobs.load(Ordering::SeqCst);
        tracing::Span::current().record("n_jobs_running", n_running);
        let min = self.min_jobs;
        println!("Tracker.next_batch_size - n_running: {n_running}, self.min_jobs: {min}");
        if n_running < self.min_jobs {
            Some(self.max_jobs - n_running)
        } else {
            None
        }
    }

    pub fn dispatch_job(&self) {
        self.running_jobs.fetch_add(1, Ordering::SeqCst);
    }

    pub fn notified(&self) -> tokio::sync::futures::Notified {
        self.notify.notified()
    }

    pub fn job_execution_inserted(&self) {
        self.notify.notify_one()
    }

    pub fn job_completed(&self, rescheduled: bool) {
        let n_running_jobs = self.running_jobs.fetch_sub(1, Ordering::SeqCst);
        println!("tracker.job_completed: previously running {n_running_jobs}");
        if rescheduled || n_running_jobs == self.min_jobs {
            self.notify.notify_one();
        }
    }
}
