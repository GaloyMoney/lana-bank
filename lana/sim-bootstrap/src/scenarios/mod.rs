mod first_payment_45d_late;
mod timely_payments;

use lana_app::{app::LanaApp, primitives::*};
use tokio::task::JoinHandle;

pub async fn run(
    sub: &Subject,
    app: &LanaApp,
) -> anyhow::Result<Vec<JoinHandle<Result<(), anyhow::Error>>>> {
    let mut handles = Vec::new();
    let sub = *sub;

    {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            timely_payments::timely_payments_scenario(sub, &app).await
        }));
    }
    {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            first_payment_45d_late::first_payment_45d_late_scenario(sub, &app).await
        }));
    }

    Ok(handles)
}
