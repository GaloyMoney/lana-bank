use anyhow::Result;

use crate::cli::DashboardAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output::{self, scalar};

pub async fn execute(
    client: &mut GraphQLClient,
    action: DashboardAction,
    json: bool,
) -> Result<()> {
    match action {
        DashboardAction::Get => {
            let vars = dashboard_get::Variables {};
            let data = client.execute::<DashboardGet>(vars).await?;
            let d = data.dashboard;
            if json {
                output::print_json(&d)?;
            } else {
                let active = d.active_facilities.to_string();
                let pending = d.pending_facilities.to_string();
                let disbursed = scalar(&d.total_disbursed);
                let collateral = scalar(&d.total_collateral);
                output::print_kv(&[
                    ("Active Facilities", &active),
                    ("Pending Facilities", &pending),
                    ("Total Disbursed", &disbursed),
                    ("Total Collateral", &collateral),
                ]);
            }
        }
    }
    Ok(())
}
