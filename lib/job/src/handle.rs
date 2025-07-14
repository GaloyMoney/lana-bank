use tokio::task::JoinHandle;

pub(crate) struct OwnedTaskHandle(Option<JoinHandle<()>>);

impl OwnedTaskHandle {
    pub fn new(inner: tokio::task::JoinHandle<()>) -> Self {
        Self(Some(inner))
    }

    pub async fn stop(self) {
        let handle = self.into_inner();
        handle.abort();
        let _ = handle.await;
    }

    fn into_inner(mut self) -> JoinHandle<()> {
        self.0.take().expect("Only consumed once")
    }
}

impl Drop for OwnedTaskHandle {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.abort();
        }
    }
}
