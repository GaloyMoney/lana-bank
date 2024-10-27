#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod event;
mod listener;
mod repo;

use serde::{de::DeserializeOwned, Serialize};
use sqlx::{postgres::PgListener, PgPool, Postgres, Transaction};
use tokio::sync::broadcast;

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

pub use event::*;
pub use listener::*;
use repo::*;

const DEFAULT_BUFFER_SIZE: usize = 100;

#[derive(Clone)]
pub(crate) struct Outbox<P>
where
    P: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    repo: OutboxRepo<P>,
    _pool: PgPool,
    event_sender: broadcast::Sender<OutboxEvent<P>>,
    event_receiver: Arc<broadcast::Receiver<OutboxEvent<P>>>,
    highest_known_sequence: Arc<AtomicU64>,
    buffer_size: usize,
}

impl<P> Outbox<P>
where
    P: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    pub(crate) async fn init(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let buffer_size = DEFAULT_BUFFER_SIZE;
        let (sender, recv) = broadcast::channel(buffer_size);
        let repo = OutboxRepo::new(pool);
        let highest_known_sequence =
            Arc::new(AtomicU64::from(repo.highest_known_sequence().await?));
        Self::spawn_pg_listener(pool, sender.clone(), Arc::clone(&highest_known_sequence)).await?;
        Ok(Self {
            event_sender: sender,
            event_receiver: Arc::new(recv),
            repo,
            highest_known_sequence,
            _pool: pool.clone(),
            buffer_size,
        })
    }

    pub(crate) async fn persist_events(
        &self,
        db: &mut Transaction<'_, Postgres>,
        events: impl IntoIterator<Item = impl Into<P>>,
    ) -> Result<(), sqlx::Error> {
        let events = self
            .repo
            .persist_events(db, events.into_iter().map(Into::into))
            .await?;

        let mut new_highest_sequence = EventSequence::BEGIN;
        for event in events {
            new_highest_sequence = event.sequence;
            let _ = self
                .event_sender
                .send(event.into())
                .map_err(|_| ())
                .expect("event receiver dropped");
        }
        self.highest_known_sequence
            .fetch_max(u64::from(new_highest_sequence), Ordering::AcqRel);
        Ok(())
    }

    pub async fn register_listener(
        &self,
        start_after: Option<EventSequence>,
    ) -> Result<OutboxListener<P>, sqlx::Error> {
        let sub = self.event_receiver.resubscribe();
        let latest_known = EventSequence::from(self.highest_known_sequence.load(Ordering::Relaxed));
        let start = start_after.unwrap_or(latest_known);
        Ok(OutboxListener::new(
            self.repo.clone(),
            sub,
            start,
            latest_known,
            self.buffer_size,
        ))
    }

    async fn spawn_pg_listener(
        pool: &PgPool,
        sender: broadcast::Sender<OutboxEvent<P>>,
        highest_known_sequence: Arc<AtomicU64>,
    ) -> Result<(), sqlx::Error> {
        let mut listener = PgListener::connect_with(pool).await?;
        listener.listen("persistent_outbox_events").await?;
        tokio::spawn(async move {
            loop {
                if let Ok(notification) = listener.recv().await {
                    if let Ok(event) =
                        serde_json::from_str::<PersistentOutboxEvent<P>>(notification.payload())
                    {
                        let new_highest_sequence = u64::from(event.sequence);
                        highest_known_sequence.fetch_max(new_highest_sequence, Ordering::AcqRel);
                        if sender.send(event.into()).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Ok(())
    }
}
