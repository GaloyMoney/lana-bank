use serde::{Serialize, de::DeserializeOwned};
use sqlx::{PgPool, Postgres, Transaction};

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

    pub async fn highest_known_sequence(&self) -> Result<EventSequence, sqlx::Error> {
        let row = sqlx::query!(
            r#"SELECT COALESCE(MAX(sequence), 0) AS "max!" FROM persistent_outbox_events"#
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(EventSequence::from(row.max as u64))
    }

    pub async fn persist_events(
        &self,
        db: &mut Transaction<'_, Postgres>,
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
        .fetch_all(&mut **db)
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
            let rows = sqlx::query!(
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
