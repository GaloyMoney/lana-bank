use sqlx::postgres::{PgListener, types::PgInterval};

use std::{sync::Arc, time::Duration};

use super::{
    config::JobExecutorConfig, handle::OwnedTaskHandle, registry::JobRegistry, repo::JobRepo,
    tracker::JobTracker,
};

pub(crate) struct NewJobExecutor {
    config: JobExecutorConfig,
    repo: JobRepo,
    registry: JobRegistry,
    tracker: Arc<JobTracker>,
}

pub(crate) struct JobExecutorHandle {
    executor: Arc<NewJobExecutor>,
    handle: OwnedTaskHandle,
}

impl NewJobExecutor {
    pub fn new(config: JobExecutorConfig, repo: JobRepo, registry: JobRegistry) -> Self {
        Self {
            tracker: Arc::new(JobTracker::new(config.min_jobs_per_process)),
            repo,
            config,
            registry,
        }
    }

    pub async fn start(self) -> Result<JobExecutorHandle, sqlx::Error> {
        let listener_handle = self.start_listener().await?;
        let executor = Arc::new(self);
        let handle = OwnedTaskHandle::new(tokio::task::spawn(Self::main_loop(
            Arc::clone(&executor),
            listener_handle,
        )));
        Ok(JobExecutorHandle { executor, handle })
    }

    fn next_batch_size(&self) -> Option<usize> {
        let n_running = self.tracker.n_jobs_running();
        if n_running < self.config.min_jobs_per_process {
            Some(self.config.max_jobs_per_process - n_running)
        } else {
            None
        }
    }

    async fn main_loop(self: Arc<Self>, _listener_task: OwnedTaskHandle) {
        let mut failures = 0;
        loop {
            let mut max_wait = Duration::from_secs(60);
            if let Some(batch_size) = self.next_batch_size() {
                // failures = 0;
            };
            let _ = crate::time::timeout(max_wait, self.tracker.notified()).await;
        }
    }

    async fn poll_and_dispatch(self: Arc<Self>) -> Result<(), sqlx::Error> {
        //
        Ok(())
    }

    async fn start_listener(&self) -> Result<OwnedTaskHandle, sqlx::Error> {
        let mut listener = PgListener::connect_with(self.repo.pool()).await?;
        listener.listen("job_execution").await?;
        let tracker = self.tracker.clone();
        Ok(OwnedTaskHandle::new(tokio::task::spawn(async move {
            loop {
                if listener.recv().await.is_ok() {
                    tracker.job_execution_inserted();
                } else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        })))
    }
}
