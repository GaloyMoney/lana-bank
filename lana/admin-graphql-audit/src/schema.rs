use async_graphql::{Context, Object, connection::*};

use super::*;

#[derive(Default)]
pub struct AuditQuery;

#[Object]
impl AuditQuery {
    async fn audit(
        &self,
        ctx: &Context<'_>,
        first: i32,
        after: Option<String>,
        subject: Option<AuditSubjectId>,
        authorized: Option<bool>,
        object: Option<String>,
        action: Option<String>,
    ) -> async_graphql::Result<Connection<AuditCursor, AuditEntry>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        let subject_filter: Option<String> = subject.map(String::from);
        let authorized_filter = authorized;
        let object_filter = object;
        let action_filter = action;
        query(
            after,
            None,
            Some(first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists");
                let res = app
                    .list_audit(
                        sub,
                        es_entity::PaginatedQueryArgs {
                            first,
                            after: after.map(lana_app::audit::AuditCursor::from),
                        },
                        subject_filter.clone(),
                        authorized_filter,
                        object_filter.clone(),
                        action_filter.clone(),
                    )
                    .await?;

                let mut connection = Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|entry| {
                        let cursor = AuditCursor::from(&entry);
                        Edge::new(cursor, AuditEntry::from(entry))
                    }));

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }

    async fn audit_subjects(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<AuditSubjectId>> {
        let (app, sub) = app_and_sub_from_ctx!(ctx);
        Ok(app
            .list_audit_subjects(sub)
            .await?
            .into_iter()
            .map(AuditSubjectId::from)
            .collect())
    }
}
