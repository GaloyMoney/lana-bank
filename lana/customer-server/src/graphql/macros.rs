/// Helper to extract the 'app' and 'sub' args
///
/// Instead of:
/// ```rust
/// async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
///     let app = ctx.data_unchecked::<LanaApp>();
///     let CustomerAuthContext { sub } = ctx.data()?;
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
        let $crate::primitives::CustomerAuthContext { sub } = $ctx.data()?;
        (app, sub)
    }};
}
