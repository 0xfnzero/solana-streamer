use tokio::task::JoinHandle;

/// Subscription handle for managing and stopping subscriptions
pub struct SubscriptionHandle {
    stream_handle: Option<JoinHandle<()>>,
    event_handle: Option<JoinHandle<()>>,
    metrics_handle: Option<JoinHandle<()>>,
}

impl SubscriptionHandle {
    /// Create a new subscription handle
    pub fn new(
        stream_handle: JoinHandle<()>,
        event_handle: Option<JoinHandle<()>>,
        metrics_handle: Option<JoinHandle<()>>,
    ) -> Self {
        Self { stream_handle: Some(stream_handle), event_handle, metrics_handle }
    }

    pub fn metrics_only(metrics_handle: Option<JoinHandle<()>>) -> Self {
        Self { stream_handle: None, event_handle: None, metrics_handle }
    }

    /// Stop subscription and abort all related tasks
    pub fn stop(self) {
        if let Some(handle) = self.stream_handle {
            handle.abort();
        }
        if let Some(handle) = self.event_handle {
            handle.abort();
        }
        if let Some(handle) = self.metrics_handle {
            handle.abort();
        }
    }

    /// Asynchronously wait for all tasks to complete
    pub async fn join(self) -> Result<(), tokio::task::JoinError> {
        if let Some(handle) = self.stream_handle {
            let _ = handle.await;
        }
        if let Some(handle) = self.event_handle {
            let _ = handle.await;
        }
        if let Some(handle) = self.metrics_handle {
            let _ = handle.await;
        }
        Ok(())
    }
}
