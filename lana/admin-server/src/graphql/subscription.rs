use std::time::Duration;

use async_graphql::*;
use futures::StreamExt;
use futures::stream::Stream;

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn test_ping(&self, message: String) -> impl Stream<Item = TestPingEvent> {
        tokio_stream::wrappers::IntervalStream::new(tokio::time::interval(Duration::from_secs(1)))
            .map(move |_| TestPingEvent {
                message: message.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
    }
}

#[derive(SimpleObject)]
pub struct TestPingEvent {
    pub message: String,
    pub timestamp: String,
}
