use serde::{Serialize, de::DeserializeOwned};
use sqlx::PgPool;

use std::collections::VecDeque;

use super::event::*;

pub(super) struct OutboxRepo<P>
where
    P: Serialize + DeserializeOwned,
{
    pool: PgPool,
    _payload: std::marker::PhantomData<P>,
}

impl<P> Clone for OutboxRepo<P>
where
    P: Serialize + DeserializeOwned,
{
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            _payload: std::marker::PhantomData,
        }
    }
}

impl<P> OutboxRepo<P>
where
    P: Serialize + DeserializeOwned + Send,
{
    pub(super) fn new(pool: &PgPool) -> Self {
        Self {
            pool: pool.clone(),
            _payload: std::marker::PhantomData,
        }
    }

    #[tracing::instrument(
        name = "outbox_lana.highest_known_sequence",
        skip_all,
        err(level = "warn")
    )]
    pub async fn highest_known_sequence(&self) -> Result<EventSequence, sqlx::Error> {
        let row = sqlx::query!(
            r#"SELECT COALESCE(MAX(sequence), 0) AS "max!" FROM persistent_outbox_events"#
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(EventSequence::from(row.max as u64))
    }

    #[tracing::instrument(name = "outbox_lana.persist_events", skip_all, err(level = "warn"))]
    pub async fn persist_events(
        &self,
        op: &mut impl es_entity::AtomicOperation,
        events: impl Iterator<Item = P>,
    ) -> Result<Vec<PersistentOutboxEvent<P>>, sqlx::Error> {
        let mut payloads = Vec::new();
        let serialized_events = events
            .map(|e| {
                let serialized_event =
                    serde_json::to_value(&e).expect("Could not serialize payload");
                payloads.push(e);
                serialized_event
            })
            .collect::<Vec<_>>();

        if payloads.is_empty() {
            return Ok(Vec::new());
        }

        let tracing_context = tracing_utils::persistence::extract();
        let tracing_json =
            serde_json::to_value(&tracing_context).expect("Could not serialize tracing context");

        let rows = sqlx::query!(
            r#"WITH new_events AS (
                 INSERT INTO persistent_outbox_events (payload, tracing_context)
                 SELECT unnest($1::jsonb[]) AS payload, $2::jsonb AS tracing_context
                 RETURNING id AS "id: OutboxEventId", sequence AS "sequence: EventSequence", recorded_at
                )
                SELECT * FROM new_events
            "#,
            &serialized_events as _,
            tracing_json
        )
        .fetch_all(op.as_executor())
        .await?;
        let events = rows
            .into_iter()
            .zip(payloads.into_iter())
            .map(|(row, payload)| PersistentOutboxEvent {
                id: row.id,
                sequence: row.sequence,
                recorded_at: row.recorded_at,
                tracing_context: Some(tracing_context.clone()),
                payload: Some(payload),
            })
            .collect::<Vec<_>>();
        Ok(events)
    }

    #[tracing::instrument(
        name = "outbox_lana.persist_ephemeral_event",
        skip_all,
        err(level = "warn")
    )]
    pub async fn persist_ephemeral_event(
        &self,
        event_type: EphemeralEventType,
        payload: P,
    ) -> Result<EphemeralOutboxEvent<P>, sqlx::Error> {
        let serialized_payload =
            serde_json::to_value(&payload).expect("Could not serialize payload");
        let tracing_context = tracing_utils::persistence::extract();
        let tracing_json =
            serde_json::to_value(&tracing_context).expect("Could not serialize tracing context");

        let row = sqlx::query!(
            r#"
            INSERT INTO ephemeral_outbox_events (event_type, payload, tracing_context)
            VALUES ($1, $2, $3)
            ON CONFLICT (event_type) DO UPDATE
            SET payload = EXCLUDED.payload,
                tracing_context = EXCLUDED.tracing_context,
                recorded_at = NOW()
            RETURNING recorded_at
            "#,
            event_type.as_str(),
            serialized_payload,
            tracing_json
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(EphemeralOutboxEvent {
            event_type,
            payload,
            tracing_context: Some(tracing_context),
            recorded_at: row.recorded_at,
        })
    }

    #[tracing::instrument(
        name = "outbox_lana.load_ephemeral_events",
        skip_all,
        err(level = "warn")
    )]
    pub async fn load_ephemeral_events(&self) -> Result<VecDeque<OutboxEvent<P>>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT event_type, payload, tracing_context, recorded_at 
            FROM ephemeral_outbox_events
            ORDER BY recorded_at
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let events = rows
            .into_iter()
            .map(|row| {
                let payload =
                    serde_json::from_value(row.payload).expect("Couldn't deserialize payload");
                let tracing_context = row.tracing_context.map(|p| {
                    serde_json::from_value(p).expect("Could not deserialize tracing context")
                });
                let event = EphemeralOutboxEvent {
                    event_type: EphemeralEventType::from(row.event_type),
                    payload,
                    tracing_context,
                    recorded_at: row.recorded_at,
                };
                OutboxEvent::from(event)
            })
            .collect::<VecDeque<_>>();
        Ok(events)
    }

    #[tracing::instrument(name = "outbox_lana.load_next_page", skip_all, err(level = "warn"))]
    pub async fn load_next_page(
        &self,
        from_sequence: EventSequence,
        buffer_size: usize,
    ) -> Result<Vec<PersistentOutboxEvent<P>>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            WITH max_sequence AS (
                SELECT COALESCE(MAX(sequence), 0) AS max FROM persistent_outbox_events
            )
            SELECT
              g.seq AS "sequence!: EventSequence",
              e.id AS "id?",
              e.payload AS "payload?",
              e.tracing_context AS "tracing_context?",
              e.recorded_at AS "recorded_at?"
            FROM
                generate_series(LEAST($1 + 1, (SELECT max FROM max_sequence)),
                  LEAST($1 + $2, (SELECT max FROM max_sequence)))
                AS g(seq)
            LEFT JOIN
                persistent_outbox_events e ON g.seq = e.sequence
            WHERE
                g.seq > $1
            ORDER BY
                g.seq ASC
            LIMIT $2"#,
            from_sequence as EventSequence,
            buffer_size as i64,
        )
        .fetch_all(&self.pool)
        .await?;
        let mut events = Vec::new();
        let mut empty_ids = Vec::new();
        for row in rows {
            if row.id.is_none() {
                empty_ids.push(row.sequence);
                continue;
            }
            events.push(PersistentOutboxEvent {
                id: OutboxEventId::from(row.id.expect("already checked")),
                sequence: row.sequence,
                payload: row
                    .payload
                    .map(|p| serde_json::from_value(p).expect("Could not deserialize payload")),
                tracing_context: row
                    .tracing_context
                    .map(|p| serde_json::from_value(p).expect("Could not deserialize payload")),
                recorded_at: row.recorded_at.unwrap_or_default(),
            });
        }

        if !empty_ids.is_empty() {
            use tracing::Instrument;
            let span = tracing::info_span!("fill_gaps", gap_cnt = empty_ids.len());
            let rows = async {
                sqlx::query!(
                    r#"
                    INSERT INTO persistent_outbox_events (sequence)
                    SELECT unnest($1::bigint[]) AS sequence
                    ON CONFLICT (sequence) DO UPDATE
                    SET sequence = EXCLUDED.sequence
                    RETURNING id, sequence AS "sequence!: EventSequence", payload, tracing_context, recorded_at
                "#,
                    &empty_ids as &[EventSequence]
                )
                .fetch_all(&self.pool)
                .await
            }
            .instrument(span)
            .await?;
            for row in rows {
                events.push(PersistentOutboxEvent {
                    id: OutboxEventId::from(row.id),
                    sequence: row.sequence,
                    payload: row
                        .payload
                        .map(|p| serde_json::from_value(p).expect("Could not deserialize payload")),
                    tracing_context: row
                        .tracing_context
                        .map(|p| serde_json::from_value(p).expect("Could not deserialize payload")),
                    recorded_at: row.recorded_at,
                });
            }
            events.sort_by(|a, b| a.sequence.cmp(&b.sequence));
        }

        Ok(events)
    }
}
