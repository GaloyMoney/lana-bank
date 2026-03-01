/// Helper to extract the 'app' and 'sub' args from the GraphQL context.
#[macro_export]
macro_rules! app_and_sub_from_ctx {
    ($ctx:expr) => {{
        let app = $ctx.data_unchecked::<lana_app::app::LanaApp>();
        let $crate::primitives::AdminAuthContext { sub } = $ctx.data()?;
        (app, sub)
    }};
}

/// Helper to do a 'standard' list_by query.
///
/// Without DataLoader (for sub-crates):
/// ```ignore
/// list_with_cursor!(Cursor, Entity, after, first, |query| app.list(sub, query))
/// ```
///
/// With DataLoader feed (for admin-server):
/// ```ignore
/// list_with_cursor!(Cursor, Entity, ctx, after, first, |query| app.list(sub, query))
/// ```
#[macro_export]
macro_rules! list_with_cursor {
    ($cursor:ty, $entity:ty, $after:expr, $first:expr, $load:expr) => {{
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
                        async_graphql::types::connection::Edge::new(cursor, <$entity>::from(entity))
                    }));

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }};
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

/// Helper to find and return an optional entity.
///
/// Without DataLoader (for sub-crates):
/// ```ignore
/// maybe_fetch_one!(Entity, app.find(sub, id))
/// ```
///
/// With DataLoader feed (for admin-server):
/// ```ignore
/// maybe_fetch_one!(Entity, ctx, app.find(sub, id))
/// ```
#[macro_export]
macro_rules! maybe_fetch_one {
    ($ty:ty, $load_entity:expr) => {
        if let Some(entity) = $load_entity.await? {
            Ok(Some(<$ty>::from(entity)))
        } else {
            Ok(None)
        }
    };
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

/// Helper to find and return a required entity (without DataLoader feed).
#[macro_export]
macro_rules! fetch_one {
    ($ty:ty, $load_entity:expr) => {{
        let entity = $load_entity.await?;
        Ok(<$ty>::from(entity))
    }};
}

/// Helper to do a 'standard' list_by query with combo cursor.
///
/// Without DataLoader (for sub-crates):
/// ```ignore
/// list_with_combo_cursor!(Cursor, Entity, sort, after, first, |query| app.list(sub, query))
/// ```
///
/// With DataLoader feed (for admin-server):
/// ```ignore
/// list_with_combo_cursor!(Cursor, Entity, sort, ctx, after, first, |query| app.list(sub, query))
/// ```
#[macro_export]
macro_rules! list_with_combo_cursor {
    ($combo_cursor:ty, $entity:ty, $sort_by:expr, $after:expr, $first:expr, $load:expr) => {{
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
                        async_graphql::types::connection::Edge::new(cursor, <$entity>::from(entity))
                    }));

                Ok::<_, async_graphql::Error>(connection)
            },
        )
        .await
    }};
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

/// Helper to execute a mutation and wrap the result in a payload.
///
/// Without DataLoader (for sub-crates):
/// ```ignore
/// exec_mutation!(Payload, Entity, app.create(sub, input))
/// ```
///
/// With DataLoader feed (for admin-server):
/// ```ignore
/// exec_mutation!(Payload, Entity, ctx, app.create(sub, input))
/// ```
#[macro_export]
macro_rules! exec_mutation {
    ($payload:ty, $ty:ty, $load:expr) => {{
        let entity = <$ty>::from($load.await?);
        Ok(<$payload>::from(entity))
    }};
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

/// Helper to create a 'standard' payload struct.
#[macro_export]
macro_rules! mutation_payload {
    ($payload:ident, $name:ident: $gql_type:ty) => {
        #[derive(async_graphql::SimpleObject)]
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
