use super::helpers;
use lana_app::{app::LanaApp, primitives::*};

pub async fn seed(sub: &Subject, app: &LanaApp) -> anyhow::Result<()> {
    let term_values = helpers::std_terms();
    app.terms_templates()
        .create_terms_template(sub, String::from("Lana Bank Terms"), term_values)
        .await?;

    Ok(())
}
