use async_trait::async_trait;

use std::{collections::HashMap, fmt, str::FromStr};

use crate::{error::AuditError, primitives::*, AuditEntry};

pub trait SystemSubject {
    fn system() -> Self;
}

#[async_trait]
pub trait AuditSvc: Clone + Sync + Send + 'static {
    type Subject: FromStr
        + fmt::Display
        + fmt::Debug
        + Clone
        + Send
        + Sync
        + SystemSubject
        + 'static;
    type Object: FromStr + fmt::Display + fmt::Debug + Copy + Send + Sync + 'static;
    type Action: FromStr + fmt::Display + fmt::Debug + Copy + Send + Sync + 'static;

    fn pool(&self) -> &sqlx::PgPool;

    async fn record_system_entry(
        &self,
        object: impl Into<Self::Object> + Send,
        action: impl Into<Self::Action> + Send,
    ) -> Result<AuditInfo, AuditError> {
        let subject = Self::Subject::system();
        let object = object.into();
        let action = action.into();

        self.record_entry(&subject, object, action, true).await
    }

    async fn record_entry(
        &self,
        subject: &Self::Subject,
        object: impl Into<Self::Object> + Send,
        action: impl Into<Self::Action> + Send,
        authorized: bool,
    ) -> Result<AuditInfo, AuditError> {
        let subject = subject.clone();
        let object = object.into();
        let action = action.into();

        let sub = subject.to_string();
        let record = sqlx::query!(
            r#"
                INSERT INTO audit_entries (subject, object, action, authorized)
                VALUES ($1, $2, $3, $4)
                RETURNING id, subject
                "#,
            &sub,
            object.to_string(),
            action.to_string(),
            authorized,
        )
        .fetch_one(self.pool())
        .await?;

        Ok(AuditInfo::from((record.id, sub)))
    }

    async fn record_system_entry_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        object: impl Into<Self::Object> + Send,
        action: impl Into<Self::Action> + Send,
    ) -> Result<AuditInfo, AuditError> {
        let subject = Self::Subject::system();
        let object = object.into();
        let action = action.into();

        self.record_entry_in_tx(tx, &subject, object, action, true)
            .await
    }

    async fn record_entry_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        subject: &Self::Subject,
        object: impl Into<Self::Object> + Send,
        action: impl Into<Self::Action> + Send,
        authorized: bool,
    ) -> Result<AuditInfo, AuditError> {
        let subject = subject.clone();
        let object = object.into();
        let action = action.into();

        let sub = subject.to_string();
        let record = sqlx::query!(
            r#"
                INSERT INTO audit_entries (subject, object, action, authorized)
                VALUES ($1, $2, $3, $4)
                RETURNING id, subject
                "#,
            &sub,
            object.to_string(),
            action.to_string(),
            authorized,
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(AuditInfo::from((record.id, sub)))
    }

    async fn list(
        &self,
        query: es_entity::PaginatedQueryArgs<AuditCursor>,
    ) -> Result<
        es_entity::PaginatedQueryRet<
            AuditEntry<Self::Subject, Self::Object, Self::Action>,
            AuditCursor,
        >,
        AuditError,
    > {
        let after_id: Option<AuditEntryId> = query.after.map(|cursor| cursor.id);
        let limit = query.first;

        let rows = sqlx::query!(
            r#"
                SELECT id AS "id: AuditEntryId", subject, object, action, authorized, recorded_at
                FROM audit_entries
                WHERE COALESCE(id < $1, true)
                ORDER BY id DESC
                LIMIT $2
                "#,
            after_id as Option<AuditEntryId>,
            (limit + 1) as i64,
        )
        .fetch_all(self.pool())
        .await?;

        let has_next_page = rows.len() > limit;

        let entries: Vec<AuditEntry<_, _, _>> = rows
            .into_iter()
            .take(limit)
            .map(|raw_event| -> Result<AuditEntry<_, _, _>, AuditError> {
                Ok(AuditEntry {
                    id: raw_event.id,
                    subject: raw_event
                        .subject
                        .parse()
                        .map_err(|_| AuditError::SubjectParseError(raw_event.subject))?,
                    object: raw_event
                        .object
                        .parse()
                        .map_err(|_| AuditError::ObjectParseError(raw_event.object))?,
                    action: raw_event
                        .action
                        .parse()
                        .map_err(|_| AuditError::ActionParseError(raw_event.action))?,
                    authorized: raw_event.authorized,
                    recorded_at: raw_event.recorded_at,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let end_cursor = if has_next_page {
            entries.last().map(|event| AuditCursor { id: event.id })
        } else {
            None
        };

        Ok(es_entity::PaginatedQueryRet {
            entities: entries,
            has_next_page,
            end_cursor,
        })
    }

    async fn find_all<T: From<AuditEntry<Self::Subject, Self::Object, Self::Action>>>(
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
        .fetch_all(self.pool())
        .await?;

        let audit_entries: HashMap<AuditEntryId, T> = raw_entries
            .into_iter()
            .map(|raw_entry| -> Result<_, AuditError> {
                let audit_entry = AuditEntry {
                    id: raw_entry.id,
                    subject: raw_entry
                        .subject
                        .parse()
                        .map_err(|_| AuditError::SubjectParseError(raw_entry.subject))?,
                    object: raw_entry
                        .object
                        .parse()
                        .map_err(|_| AuditError::ObjectParseError(raw_entry.object))?,
                    action: raw_entry
                        .action
                        .parse()
                        .map_err(|_| AuditError::ActionParseError(raw_entry.action))?,
                    authorized: raw_entry.authorized,
                    recorded_at: raw_entry.recorded_at,
                };
                Ok((raw_entry.id, T::from(audit_entry)))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(audit_entries)
    }
}
