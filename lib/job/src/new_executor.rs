use std::sync::Arc;

use super::{handle::OwnedTaskHandle, tracker::JobTracker};

pub(crate) struct NewJobExecutor {
    tracker: Arc<JobTracker>,
}

pub(crate) struct JobExecutorHandle {
    runner: Arc<NewJobExecutor>,
    handle: Option<OwnedTaskHandle>,
}
