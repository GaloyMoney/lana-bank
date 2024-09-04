use std::collections::HashMap;

use chrono::{DateTime, Utc};

pub mod error;
use error::AuditError;

mod cursor;
pub use cursor::AuditCursor;

use crate::{
    authorization::{Action, Object},
    primitives::{AuditEntryId, AuditInfo, Subject},
};

pub struct AuditEntry {
    pub id: AuditEntryId,
    pub subject: Subject,
    pub object: Object,
    pub action: Action,
    pub authorized: bool,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct Audit {
    pool: sqlx::PgPool,
}

impl Audit {
    pub fn new(pool: &sqlx::PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn record_entry(
        &self,
        subject: &Subject,
        object: Object,
        action: impl Into<Action>,
        authorized: bool,
    ) -> Result<AuditInfo, AuditError> {
        let mut db = self.pool.begin().await?;
        let info = self
            .record_entry_in_tx(&mut db, subject, object, action, authorized)
            .await?;
        db.commit().await?;
        Ok(info)
    }

    pub async fn record_entry_in_tx(
        &self,
        db: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        subject: &Subject,
        object: Object,
        action: impl Into<Action>,
        authorized: bool,
    ) -> Result<AuditInfo, AuditError> {
        let action = action.into();
        let record = sqlx::query!(
            r#"
                INSERT INTO audit_entries (subject, object, action, authorized)
                VALUES ($1, $2, $3, $4)
                RETURNING id, subject
                "#,
            subject.to_string(),
            object.to_string(),
            action.to_string(),
            authorized,
        )
        .fetch_one(&mut **db)
        .await?;

        Ok(AuditInfo::from((record.id, *subject)))
    }

    pub async fn list(
        &self,
        query: crate::query::PaginatedQueryArgs<AuditCursor>,
    ) -> Result<crate::query::PaginatedQueryRet<AuditEntry, AuditCursor>, AuditError> {
        let after_id: Option<AuditEntryId> = query.after.map(|cursor| cursor.id);
        let limit = i64::try_from(query.first)?;

        let raw_events = sqlx::query!(
            r#"
            SELECT id AS "id: AuditEntryId", subject, object, action, authorized, recorded_at
            FROM audit_entries
            WHERE ($1::BIGINT IS NULL OR id < $1::BIGINT)
            ORDER BY id DESC
            LIMIT $2
            "#,
            after_id as Option<AuditEntryId>,
            limit + 1,
        )
        .fetch_all(&self.pool)
        .await?;

        // Determine if there is a next page
        let has_next_page = raw_events.len() as i64 > limit;

        // If we fetched one extra, remove it from the results
        let events = if has_next_page {
            raw_events
                .into_iter()
                .take(limit.try_into().expect("can't convert to usize"))
                .collect()
        } else {
            raw_events
        };

        // Create the next cursor if there is a next page
        let end_cursor = if has_next_page {
            events.last().map(|event| AuditCursor { id: event.id })
        } else {
            None
        };

        let audit_entries: Vec<AuditEntry> = events
            .into_iter()
            .map(|raw_event| AuditEntry {
                id: raw_event.id,
                subject: raw_event.subject.parse().expect("Could not parse subject"),
                object: raw_event.object.parse().expect("Could not parse object"),
                action: raw_event.action.parse().expect("Could not parse action"),
                authorized: raw_event.authorized,
                recorded_at: raw_event.recorded_at,
            })
            .collect();

        Ok(crate::query::PaginatedQueryRet {
            entities: audit_entries,
            has_next_page,
            end_cursor,
        })
    }

    pub async fn find_all<T: From<AuditEntry>>(
        &self,
        ids: &[AuditEntryId],
    ) -> Result<HashMap<AuditEntryId, T>, AuditError> {
        let raw_entries = sqlx::query!(
            r#"
            SELECT id AS "id: AuditEntryId", subject, object, action, authorized, recorded_at
            FROM audit_entries
            WHERE id = ANY($1)
            "#,
            &ids as &[AuditEntryId],
        )
        .fetch_all(&self.pool)
        .await?;

        let audit_entries: HashMap<AuditEntryId, T> = raw_entries
            .into_iter()
            .map(|raw_entry| {
                let audit_entry = AuditEntry {
                    id: raw_entry.id,
                    subject: raw_entry.subject.parse().expect("Could not parse subject"),
                    object: raw_entry.object.parse().expect("Could not parse object"),
                    action: raw_entry.action.parse().expect("Could not parse action"),
                    authorized: raw_entry.authorized,
                    recorded_at: raw_entry.recorded_at,
                };
                (raw_entry.id, T::from(audit_entry))
            })
            .collect();

        Ok(audit_entries)
    }
}
