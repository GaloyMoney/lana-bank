/// Helper to extract the 'app' and 'sub' args
///
/// Instead of:
/// ```rust
/// async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
///     let app = ctx.data_unchecked::<LanaApp>();
///     let AdminAuthContext { sub } = ctx.data()?;
/// ```
///
/// Use:
/// ```rust
/// async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
///     let (app, sub) = app_and_sub_from_ctx!(ctx);
/// ```
#[macro_export]
macro_rules! app_and_sub_from_ctx {
    ($ctx:expr) => {{
        let app = $ctx.data_unchecked::<lana_app::app::LanaApp>();
        let $crate::primitives::AdminAuthContext { sub } = $ctx.data()?;
        (app, sub)
    }};
}

#[macro_export]
macro_rules! loader_and_sub_from_ctx {
    ($ctx:expr) => {{
        let loader = $ctx.data_unchecked::<LanaDataLoader>();
        let $crate::primitives::AdminAuthContext { sub } = $ctx.data()?;
        (loader, sub)
    }};
}

/// Helper for a 'standard' find and return
///
/// Instead of:
/// ```rust
/// if let Some(domain_user) = app.users().find_by_id(sub, id).await? {
///     let user = User::from(domain_user)
///     let loader = ctx.data_unchecked::<LanaDataLoader>();
///     loader.feed_one(user.entity.id, user.clone()).await;
///     Ok(Some(user))
/// } else {
///     Ok(None)
/// }
/// ```
///
/// Use:
/// ```rust
/// maybe_fetch_one!(User, ctx, app.users().find_by_id(sub, id).await?)
/// ```
#[macro_export]
macro_rules! maybe_fetch_one {
    ($ty:ty, $ctx:expr, $load_entity:expr) => {
        if let Some(entity) = $load_entity.await? {
            let entity = <$ty>::from(entity);
            let loader = $ctx.data_unchecked::<LanaDataLoader>();
            loader.feed_one(entity.entity.id, entity.clone()).await;
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    };
    ($ty:ty, $id:ty, $ctx:expr, $load_entity:expr) => {
        if let Some(entity) = $load_entity.await? {
            let entity = <$ty>::from(entity);
            let loader = $ctx.data_unchecked::<LanaDataLoader>();
            loader
                .feed_one(<$id>::from(entity.entity.id), entity.clone())
                .await;
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    };
}

#[macro_export]
macro_rules! exec_mutation {
    ($payload:ty, $ty:ty, $ctx:expr, $load:expr) => {{
        let entity = <$ty>::from($load.await?);
        let loader = $ctx.data_unchecked::<LanaDataLoader>();
        loader.feed_one(entity.entity.id, entity.clone()).await;
        Ok(<$payload>::from(entity))
    }};
    ($payload:ty, $ty:ty, $id:ty, $ctx:expr, $load:expr) => {{
        let entity = <$ty>::from($load.await?);
        let loader = $ctx.data_unchecked::<LanaDataLoader>();
        loader
            .feed_one(<$id>::from(entity.entity.id), entity.clone())
            .await;
        Ok(<$payload>::from(entity))
    }};
}

/// Helper to do a 'standard' list_by query
///
/// Example usage:
/// ```rust
/// query(
///     after,
///     None,
///     Some(first),
///     None,
///     |after, _, first, _| async move {
///         let first = first.expect("First always exists");
///         let res = app
///             .governance()
///             .list_committees(
///                 sub,
///                 es_entity::PaginatedQueryArgs {
///                     first,
///                     after: after.map(
///                         governance::committee_cursor::CommitteeByCreatedAtCursor::from,
///                     ),
///                 },
///             )
///             .await?;
///
///         let mut connection = Connection::new(false, res.has_next_page);
///         connection
///             .edges
///             .extend(res.entities.into_iter().map(|committee| {
///                 let cursor = CommitteeByCreatedAtCursor::from(&committee);
///                 Edge::new(cursor, Committee::from(committee))
///             }));
///
///         Ok::<_, async_graphql::Error>(connection)
///     },
/// )
/// .await
/// ```
#[macro_export]
macro_rules! list_with_cursor {
    ($cursor:ty, $entity:ty, $ctx:expr, $after:expr, $first:expr, $load:expr) => {{
        let loader = $ctx.data_unchecked::<LanaDataLoader>();
        async_graphql::types::connection::query(
            $after,
            None,
            Some($first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists") as usize;
                let args = es_entity::PaginatedQueryArgs { first, after };
                let res = $load(args).await?;
                let mut connection =
                    async_graphql::types::connection::Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|entity| {
                        let cursor = <$cursor>::from(&entity);
                        Edge::new(cursor, <$entity>::from(entity))
                    }));
                loader
                    .feed_many(
                        connection
                            .edges
                            .iter()
                            .map(|e| (e.node.entity.id, e.node.clone())),
                    )
                    .await;

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }};
}

/// Helper to do a 'standard' list_by query with combo cursor
///
/// Example usage:
/// ```rust
/// query(
///     after,
///     None,
///     Some(first),
///     None,
///     |after, _, first, _| async move {
///         let first = first.expect("First always exists");
///         let after = after
///             .map(CommitteeCursor::from);
///
///         let res = app
///             .governance()
///             .list_committees(
///                 sub,
///                 es_entity::PaginatedQueryArgs {
///                     first,
///                     after,
///                 },
///                 filter,
///                 sort,
///             )
///             .await?;
///
///         let mut connection = Connection::new(false, res.has_next_page);
///         connection
///             .edges
///             .extend(res.entities.into_iter().map(|committee| {
///                 let cursor = CommitteeCursor::from((
///                     DomainCommitteesCursorSortBy::from(sort),
///                     &committee,
///                 ));
///                 Edge::new(cursor, Committee::from(committee))
///             }));
///
///         Ok::<_, async_graphql::Error>(connection)
///     },
/// )
/// .await
/// ```
#[macro_export]
macro_rules! list_with_combo_cursor {
    ($combo_cursor:ty, $entity:ty, $sort_by:expr, $ctx:expr, $after:expr, $first:expr, $load:expr) => {{
        let loader = $ctx.data_unchecked::<LanaDataLoader>();
        async_graphql::types::connection::query(
            $after,
            None,
            Some($first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists") as usize;
                let after = after.map(<$combo_cursor>::from);
                let args = es_entity::PaginatedQueryArgs { first, after };
                let res = $load(args).await?;
                let mut connection =
                    async_graphql::types::connection::Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|entity| {
                        let cursor = <$combo_cursor>::from(($sort_by, &entity));
                        Edge::new(cursor, <$entity>::from(entity))
                    }));
                loader
                    .feed_many(
                        connection
                            .edges
                            .iter()
                            .map(|e| (e.node.entity.id, e.node.clone())),
                    )
                    .await;

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }};
}

/// Like `maybe_fetch_one!` but feeds the loader with `(Subject, Id)` keys
/// for authorization-aware data loading. Extracts `sub` from `ctx` for the
/// loader key; the caller's `$load_entity` expression should use its own `sub`.
#[macro_export]
macro_rules! authz_maybe_fetch_one {
    ($ty:ty, $ctx:expr, $load_entity:expr) => {
        if let Some(entity) = $load_entity.await? {
            let entity = <$ty>::from(entity);
            let loader = $ctx.data_unchecked::<LanaDataLoader>();
            let $crate::primitives::AdminAuthContext { sub: __sub } = $ctx.data()?;
            loader
                .feed_one((__sub.clone(), entity.entity.id), entity.clone())
                .await;
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    };
}

/// Like `exec_mutation!` but feeds the loader with `(Subject, Id)` keys
/// for authorization-aware data loading. Extracts `sub` from `ctx` for the
/// loader key; the caller's `$load` expression should use its own `sub`.
#[macro_export]
macro_rules! authz_exec_mutation {
    ($payload:ty, $ty:ty, $ctx:expr, $load:expr) => {{
        let entity = <$ty>::from($load.await?);
        let loader = $ctx.data_unchecked::<LanaDataLoader>();
        let $crate::primitives::AdminAuthContext { sub: __sub } = $ctx.data()?;
        loader
            .feed_one((__sub.clone(), entity.entity.id), entity.clone())
            .await;
        Ok(<$payload>::from(entity))
    }};
}

/// Like `list_with_combo_cursor!` but feeds the loader with `(Subject, Id)` keys
/// for authorization-aware data loading. Extracts `sub` from `ctx` for the
/// loader key; the caller's `$load` closure should use its own `sub`.
#[macro_export]
macro_rules! authz_list_with_combo_cursor {
    ($combo_cursor:ty, $entity:ty, $sort_by:expr, $ctx:expr, $after:expr, $first:expr, $load:expr) => {{
        let loader = $ctx.data_unchecked::<LanaDataLoader>();
        let $crate::primitives::AdminAuthContext { sub: __sub } = $ctx.data()?;
        let __sub = __sub.clone();
        async_graphql::types::connection::query(
            $after,
            None,
            Some($first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists") as usize;
                let after = after.map(<$combo_cursor>::from);
                let args = es_entity::PaginatedQueryArgs { first, after };
                let res = $load(args).await?;
                let mut connection =
                    async_graphql::types::connection::Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|entity| {
                        let cursor = <$combo_cursor>::from(($sort_by, &entity));
                        Edge::new(cursor, <$entity>::from(entity))
                    }));
                loader
                    .feed_many(
                        connection
                            .edges
                            .iter()
                            .map(|e| ((__sub.clone(), e.node.entity.id), e.node.clone())),
                    )
                    .await;

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }};
}

/// Helper to create a 'standard' payload
///
/// Creates a payload struct of the form:
/// ```rust
/// pub struct SomeMutationPayload {
///     entity: Entity
/// }
/// ```
///
/// Example usage:
/// ```rust
/// mutation_payload! { PolicyAssignCommitteePayload, policy: Policy }
/// ```
#[macro_export]
macro_rules! mutation_payload {
    ($payload:ident, $name:ident: $gql_type:ty) => {
        #[derive(SimpleObject)]
        pub struct $payload {
            $name: $gql_type,
        }

        impl From<$gql_type> for $payload {
            fn from($name: $gql_type) -> Self {
                Self { $name }
            }
        }
    };
}

#[macro_export]
macro_rules! list_with_cursor_and_id {
    ($cursor:ty, $entity:ty, $id:ty, $ctx:expr, $after:expr, $first:expr, $load:expr) => {{
        let loader = $ctx.data_unchecked::<LanaDataLoader>();
        async_graphql::types::connection::query(
            $after,
            None,
            Some($first),
            None,
            |after, _, first, _| async move {
                let first = first.expect("First always exists") as usize;
                let args = es_entity::PaginatedQueryArgs { first, after };
                let res = $load(args).await?;
                let mut connection =
                    async_graphql::types::connection::Connection::new(false, res.has_next_page);
                connection
                    .edges
                    .extend(res.entities.into_iter().map(|entity| {
                        let cursor = <$cursor>::from(&entity);
                        Edge::new(cursor, <$entity>::from(entity))
                    }));
                loader
                    .feed_many(
                        connection
                            .edges
                            .iter()
                            .map(|e| (<$id>::from(e.node.entity.id), e.node.clone())),
                    )
                    .await;

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }};
}
