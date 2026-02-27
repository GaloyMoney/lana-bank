use anyhow::Result;

use crate::cli::LiquidationAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output::{self, scalar, sval};

pub async fn execute(
    client: &mut GraphQLClient,
    action: LiquidationAction,
    json: bool,
) -> Result<()> {
    match action {
        LiquidationAction::Find { id } => {
            let vars = find_liquidation::Variables { id };
            let data = client.execute::<FindLiquidation>(vars).await?;
            match data.liquidation {
                Some(l) => {
                    if json {
                        output::print_json(&l)?;
                    } else {
                        let amount_received = scalar(&l.amount_received);
                        let proceeds: Vec<String> = l
                            .received_proceeds
                            .iter()
                            .map(|p| scalar(&p.amount))
                            .collect();
                        output::print_kv(&[
                            ("Liquidation ID", &l.liquidation_id),
                            ("Amount Received", &amount_received),
                            ("Received Proceeds", &format!("{:?}", proceeds)),
                        ]);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Liquidation not found");
                    }
                }
            }
        }
        LiquidationAction::RecordCollateralSent {
            collateral_id,
            amount,
        } => {
            let vars = liquidation_record_collateral_sent::Variables {
                input: liquidation_record_collateral_sent::CollateralRecordSentToLiquidationInput {
                    collateral_id,
                    amount: sval(amount),
                },
            };
            let data = client
                .execute::<LiquidationRecordCollateralSent>(vars)
                .await?;
            let c = data.collateral_record_sent_to_liquidation.collateral;
            if json {
                output::print_json(&c)?;
            } else {
                let liquidations_info: String = match &c.credit_facility {
                    Some(cf) => cf
                        .liquidations
                        .iter()
                        .map(|l| format!("{} (sent: {})", l.liquidation_id, scalar(&l.sent_total)))
                        .collect::<Vec<_>>()
                        .join(", "),
                    None => "N/A".to_string(),
                };
                output::print_kv(&[
                    ("Collateral ID", &c.collateral_id),
                    ("Liquidations", &liquidations_info),
                ]);
            }
        }
        LiquidationAction::RecordPaymentReceived {
            collateral_id,
            amount,
        } => {
            let vars = liquidation_record_payment_received::Variables {
                input:
                    liquidation_record_payment_received::CollateralRecordProceedsFromLiquidationInput {
                        collateral_id,
                        amount: sval(amount),
                    },
            };
            let data = client
                .execute::<LiquidationRecordPaymentReceived>(vars)
                .await?;
            let c = data.collateral_record_proceeds_from_liquidation.collateral;
            if json {
                output::print_json(&c)?;
            } else {
                output::print_kv(&[("Collateral ID", &c.collateral_id)]);
            }
        }
    }
    Ok(())
}
