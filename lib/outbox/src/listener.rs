use futures::{FutureExt, Stream, StreamExt};
use serde::{Serialize, de::DeserializeOwned};
use tokio::{sync::broadcast, task::JoinHandle};
use tokio_stream::wrappers::{BroadcastStream, errors::BroadcastStreamRecvError};

use std::{
    collections::{BTreeMap, VecDeque},
    pin::Pin,
    task::Poll,
};

use super::{event::*, repo::*};

pub struct OutboxListener<P>
where
    P: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    repo: OutboxRepo<P>,
    last_returned_sequence: EventSequence,
    latest_known: EventSequence,
    event_receiver: Pin<Box<BroadcastStream<OutboxEvent<P>>>>,
    buffer_size: usize,
    sequenced_cache: BTreeMap<EventSequence, OutboxEvent<P>>,
    return_queue: VecDeque<OutboxEvent<P>>,
    next_page_handle: Option<JoinHandle<Result<Vec<PersistentOutboxEvent<P>>, sqlx::Error>>>,
}

impl<P> OutboxListener<P>
where
    P: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    pub(super) fn new(
        repo: OutboxRepo<P>,
        event_receiver: broadcast::Receiver<OutboxEvent<P>>,
        start_after: EventSequence,
        latest_known: EventSequence,
        buffer: usize,
        current_ephemeral_events: VecDeque<OutboxEvent<P>>,
    ) -> Self {
        Self {
            repo,
            last_returned_sequence: start_after,
            latest_known,
            event_receiver: Box::pin(BroadcastStream::new(event_receiver)),
            sequenced_cache: BTreeMap::new(),
            return_queue: current_ephemeral_events,
            next_page_handle: None,
            buffer_size: buffer,
        }
    }

    fn maybe_add_to_cache(&mut self, event: impl Into<OutboxEvent<P>>) {
        let event = event.into();
        match event {
            OutboxEvent::Persistent(ref persistent_event) => {
                self.latest_known = self.latest_known.max(persistent_event.sequence);

                if persistent_event.sequence > self.last_returned_sequence
                    && self
                        .sequenced_cache
                        .insert(persistent_event.sequence, event)
                        .is_none()
                    && self.sequenced_cache.len() > self.buffer_size
                {
                    self.sequenced_cache.pop_last();
                }
            }
            event => {
                self.return_queue.push_back(event);
            }
        }
    }
}

impl<P> Stream for OutboxListener<P>
where
    P: Serialize + DeserializeOwned + Send + Sync + 'static + Unpin,
{
    type Item = OutboxEvent<P>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();

        // Poll page if present
        if let Some(fetch) = this.next_page_handle.as_mut() {
            match fetch.poll_unpin(cx) {
                Poll::Ready(Ok(Ok(events))) => {
                    for event in events {
                        this.maybe_add_to_cache(event);
                    }
                    this.next_page_handle = None;
                }
                Poll::Ready(_) => {
                    this.next_page_handle = None;
                }
                Poll::Pending => (),
            }
        }
        // Poll as many events as we can come
        loop {
            match this.event_receiver.as_mut().poll_next(cx) {
                Poll::Ready(None) => {
                    if let Some(handle) = this.next_page_handle.take() {
                        handle.abort();
                    }
                    return Poll::Ready(None);
                }
                Poll::Ready(Some(Ok(event))) => {
                    this.maybe_add_to_cache(event);
                }
                Poll::Ready(Some(Err(BroadcastStreamRecvError::Lagged(_)))) => (),
                Poll::Pending => break,
            }
        }

        while let Some((seq, event)) = this.sequenced_cache.pop_first() {
            if seq <= this.last_returned_sequence {
                continue;
            }
            if seq == this.last_returned_sequence.next() {
                this.last_returned_sequence = seq;
                if let Some(handle) = this.next_page_handle.take() {
                    handle.abort();
                }
                this.return_queue.push_back(event);
                break;
            }
            this.sequenced_cache.insert(seq, event);
            break;
        }

        if let Some(ret) = this.return_queue.pop_front() {
            return Poll::Ready(Some(ret));
        }

        if this.next_page_handle.is_none() && this.last_returned_sequence < this.latest_known {
            let repo = this.repo.clone();
            let last_sequence = this.last_returned_sequence;
            let buffer_size = this.buffer_size;
            this.next_page_handle = Some(tokio::spawn(async move {
                repo.load_next_page(last_sequence, buffer_size)
                    .await
                    .map_err(|e| {
                        tracing::error!(error = %e, "error loading next page");
                        e
                    })
            }));
            return this.poll_next_unpin(cx);
        }
        Poll::Pending
    }
}
