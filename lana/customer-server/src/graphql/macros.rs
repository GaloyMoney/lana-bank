// Helper to extract the 'app' and 'sub' args
#[macro_export]
macro_rules! app_and_sub_from_ctx {
    ($ctx:expr) => {{
        let app = $ctx.data_unchecked::<lana_app::app::LanaApp>();
        let $crate::primitives::CustomerAuthContext { sub } = $ctx.data()?;
        (app, sub)
    }};
}
