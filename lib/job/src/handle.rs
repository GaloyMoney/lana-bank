pub(crate) struct OwnedTaskHandle(Option<tokio::task::JoinHandle<()>>);

impl OwnedTaskHandle {
    pub fn new(inner: tokio::task::JoinHandle<()>) -> Self {
        Self(Some(inner))
    }
}

impl Drop for OwnedTaskHandle {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.abort();
        }
    }
}
