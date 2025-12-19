#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

pub mod error;
mod event;
mod listener;
mod repo;

use futures::{StreamExt, stream::BoxStream};
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{PgPool, postgres::PgListener};
use tokio::sync::broadcast;
use tracing_macros::record_error_severity;

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use error::*;
pub use event::*;
pub use listener::*;
use repo::*;

const DEFAULT_BUFFER_SIZE: usize = 100;

pub struct Outbox<P>
where
    P: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    repo: OutboxRepo<P>,
    event_sender: broadcast::Sender<OutboxEvent<P>>,
    event_receiver: Arc<broadcast::Receiver<OutboxEvent<P>>>,
    highest_known_sequence: Arc<AtomicU64>,
    buffer_size: usize,
}

impl<P> Clone for Outbox<P>
where
    P: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
            event_sender: self.event_sender.clone(),
            event_receiver: self.event_receiver.clone(),
            highest_known_sequence: self.highest_known_sequence.clone(),
            buffer_size: self.buffer_size,
        }
    }
}

impl<P> Outbox<P>
where
    P: Serialize + DeserializeOwned + Send + Sync + 'static + Unpin,
{
    #[record_error_severity]
    #[tracing::instrument(name = "outbox.init", skip(pool), fields(highest_sequence = tracing::field::Empty))]
    pub async fn init(pool: &PgPool) -> Result<Self, OutboxError> {
        let buffer_size = DEFAULT_BUFFER_SIZE;
        let (sender, recv) = broadcast::channel(buffer_size);
        let repo = OutboxRepo::new(pool);
        let highest_known_sequence =
            Arc::new(AtomicU64::from(repo.highest_known_sequence().await?));

        let seq = highest_known_sequence.load(Ordering::Relaxed);
        tracing::Span::current().record("highest_sequence", seq);

        Self::spawn_pg_listeners(pool, sender.clone(), Arc::clone(&highest_known_sequence)).await?;
        Ok(Self {
            event_sender: sender,
            event_receiver: Arc::new(recv),
            repo,
            highest_known_sequence,
            buffer_size,
        })
    }

    #[record_error_severity]
    #[tracing::instrument(name = "outbox.publish_persisted", skip_all)]
    pub async fn publish_persisted(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        event: impl Into<P>,
    ) -> Result<(), OutboxError> {
        self.publish_all_persisted(op, std::iter::once(event)).await
    }

    #[record_error_severity]
    #[tracing::instrument(name = "outbox.publish_all_persisted", skip_all)]
    pub async fn publish_all_persisted(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        events: impl IntoIterator<Item = impl Into<P>>,
    ) -> Result<(), OutboxError> {
        let _ = self
            .repo
            .persist_events(op, events.into_iter().map(Into::into))
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[tracing::instrument(name = "outbox.publish_ephemeral", skip_all)]
    pub async fn publish_ephemeral(
        &self,
        event_type: EphemeralEventType,
        event: impl Into<P>,
    ) -> Result<(), OutboxError> {
        self.repo
            .persist_ephemeral_event(event_type, event.into())
            .await?;
        Ok(())
    }

    #[record_error_severity]
    #[tracing::instrument(name = "outbox.listen_all", skip(self), fields(start_after = ?start_after, latest_known = tracing::field::Empty))]
    pub async fn listen_all(
        &self,
        start_after: Option<EventSequence>,
    ) -> Result<OutboxListener<P>, OutboxError> {
        let sub = self.event_receiver.resubscribe();
        let latest_known = EventSequence::from(self.highest_known_sequence.load(Ordering::Relaxed));
        tracing::Span::current().record("latest_known", u64::from(latest_known));

        let start = start_after.unwrap_or(latest_known);
        let current_ephemeral_events = self.repo.load_ephemeral_events().await?;
        Ok(OutboxListener::new(
            self.repo.clone(),
            sub,
            start,
            latest_known,
            self.buffer_size,
            current_ephemeral_events,
        ))
    }

    #[tracing::instrument(name = "outbox.listen_persisted", skip(self), fields(start_after = ?start_after), err)]
    pub async fn listen_persisted(
        &self,
        start_after: Option<EventSequence>,
    ) -> Result<BoxStream<'_, Arc<PersistentOutboxEvent<P>>>, OutboxError> {
        let listener = self.listen_all(start_after);
        Ok(Box::pin(listener.filter_map(|event| async move {
            match event {
                OutboxEvent::Persistent(persistent_event) => Some(persistent_event),
                _ => None,
            }
        })))
    }

    #[tracing::instrument(name = "outbox.listen_ephemeral", skip(self), err)]
    pub async fn listen_ephemeral(
        &self,
    ) -> Result<BoxStream<'_, Arc<EphemeralOutboxEvent<P>>>, OutboxError> {
        let listener = self.listen_all(None);
        Ok(Box::pin(listener.filter_map(|event| async move {
            match event {
                OutboxEvent::Ephemeral(event) => Some(event),
                _ => None,
            }
        })))
    }

    #[record_error_severity]
    #[tracing::instrument(name = "outbox.spawn_pg_listener", skip_all)]
    async fn spawn_pg_listeners(
        pool: &PgPool,
        sender: broadcast::Sender<OutboxEvent<P>>,
        highest_known_sequence: Arc<AtomicU64>,
    ) -> Result<(), OutboxError> {
        let mut listener = PgListener::connect_with(pool).await?;
        listener.listen("persistent_outbox_events").await?;
        let persistent_sender = sender.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(notification) = listener.recv().await
                    && let Ok(event) =
                        serde_json::from_str::<PersistentOutboxEvent<P>>(notification.payload())
                {
                    let new_highest_sequence = u64::from(event.sequence);
                    highest_known_sequence.fetch_max(new_highest_sequence, Ordering::AcqRel);
                    if persistent_sender.send(event.into()).is_err() {
                        break;
                    }
                }
            }
        });

        let mut listener = PgListener::connect_with(pool).await?;
        listener.listen("ephemeral_outbox_events").await?;
        tokio::spawn(async move {
            loop {
                if let Ok(notification) = listener.recv().await
                    && let Ok(event) =
                        serde_json::from_str::<EphemeralOutboxEvent<P>>(notification.payload())
                    && sender.send(event.into()).is_err()
                {
                    break;
                }
            }
        });
        Ok(())
    }
}
