// Helper to extract the 'app' and 'sub' args
// instead of:
//
// async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
//     let app = ctx.data_unchecked::<LanaApp>();
//     let AdminAuthContext { sub } = ctx.data()?;
//
// use
//
// async fn users(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<User>> {
//     let (app, sub) = app_and_sub_from_ctx!(ctx);
//
#[macro_export]
macro_rules! app_and_sub_from_ctx {
    ($ctx:expr) => {{
        let app = $ctx.data_unchecked::<lana_app::app::LanaApp>();
        let $crate::primitives::CustomerAuthContext { sub } = $ctx.data()?;
        (app, sub)
    }};
}
