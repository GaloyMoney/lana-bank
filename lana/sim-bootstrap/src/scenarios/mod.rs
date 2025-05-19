mod disbursal_different_months;
mod interest_under_payment;
mod principal_late;
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
            principal_late::principal_late_scenario(sub, &app).await
        }));
    }
    {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            disbursal_different_months::disbursal_different_months_scenario(sub, &app).await
        }));
    }
    {
        let app = app.clone();
        handles.push(tokio::spawn(async move {
            interest_under_payment::interest_under_payment_scenario(sub, &app).await
        }));
    }

    Ok(handles)
}
