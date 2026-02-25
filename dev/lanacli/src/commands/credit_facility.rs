use anyhow::Result;

use crate::cli::CreditFacilityAction;
use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output::{self, scalar, sval};

pub async fn execute(
    client: &mut GraphQLClient,
    action: CreditFacilityAction,
    json: bool,
) -> Result<()> {
    match action {
        CreditFacilityAction::ProposalCreate {
            customer_id,
            facility_amount,
            custodian_id,
            annual_rate,
            accrual_interval,
            accrual_cycle_interval,
            one_time_fee_rate,
            disbursal_policy,
            duration_months,
            initial_cvl,
            margin_call_cvl,
            liquidation_cvl,
            interest_due_days,
            overdue_days,
            liquidation_days,
        } => {
            let vars = credit_facility_proposal_create::Variables {
                input: credit_facility_proposal_create::CreditFacilityProposalCreateInput {
                    customer_id,
                    facility: sval(facility_amount),
                    custodian_id,
                    terms: credit_facility_proposal_create::TermsInput {
                        annual_rate: sval(annual_rate),
                        accrual_interval: parse_interest_interval(&accrual_interval)?,
                        accrual_cycle_interval: parse_interest_interval(&accrual_cycle_interval)?,
                        one_time_fee_rate: sval(one_time_fee_rate),
                        disbursal_policy: parse_disbursal_policy(&disbursal_policy)?,
                        duration: credit_facility_proposal_create::DurationInput {
                            period: credit_facility_proposal_create::Period::MONTHS,
                            units: duration_months,
                        },
                        initial_cvl: sval(initial_cvl),
                        margin_call_cvl: sval(margin_call_cvl),
                        liquidation_cvl: sval(liquidation_cvl),
                        interest_due_duration_from_accrual:
                            credit_facility_proposal_create::DurationInput {
                                period: credit_facility_proposal_create::Period::DAYS,
                                units: interest_due_days,
                            },
                        obligation_overdue_duration_from_due:
                            credit_facility_proposal_create::DurationInput {
                                period: credit_facility_proposal_create::Period::DAYS,
                                units: overdue_days,
                            },
                        obligation_liquidation_duration_from_due:
                            credit_facility_proposal_create::DurationInput {
                                period: credit_facility_proposal_create::Period::DAYS,
                                units: liquidation_days,
                            },
                    },
                },
            };
            let data = client.execute::<CreditFacilityProposalCreate>(vars).await?;
            let p = data
                .credit_facility_proposal_create
                .credit_facility_proposal;
            if json {
                output::print_json(&p)?;
            } else {
                let amount = scalar(&p.facility_amount);
                let rate = scalar(&p.credit_facility_terms.annual_rate);
                output::print_kv(&[
                    ("Proposal ID", &p.credit_facility_proposal_id),
                    ("Status", &format!("{:?}", p.status)),
                    ("Facility Amount", &amount),
                    ("Annual Rate", &rate),
                    (
                        "Duration",
                        &format!(
                            "{} {:?}",
                            p.credit_facility_terms.duration.units,
                            p.credit_facility_terms.duration.period,
                        ),
                    ),
                    ("Created", &p.created_at),
                ]);
            }
        }
        CreditFacilityAction::ProposalList { first, after } => {
            let vars = credit_facility_proposals_list::Variables { first, after };
            let data = client.execute::<CreditFacilityProposalsList>(vars).await?;
            let nodes = data.credit_facility_proposals.nodes;
            if json {
                output::print_json(&nodes)?;
            } else {
                let rows: Vec<Vec<String>> = nodes
                    .iter()
                    .map(|p| {
                        vec![
                            p.credit_facility_proposal_id.clone(),
                            format!("{:?}", p.status),
                            scalar(&p.facility_amount),
                            p.created_at.clone(),
                        ]
                    })
                    .collect();
                output::print_table(&["Proposal ID", "Status", "Amount", "Created"], rows);
                let pi = data.credit_facility_proposals.page_info;
                if pi.has_next_page
                    && let Some(cursor) = pi.end_cursor
                {
                    println!("\nMore results available. Use --after {cursor}");
                }
            }
        }
        CreditFacilityAction::List { first, after } => {
            let vars = credit_facilities_list::Variables { first, after };
            let data = client.execute::<CreditFacilitiesList>(vars).await?;
            let nodes = data.credit_facilities.nodes;
            if json {
                output::print_json(&nodes)?;
            } else {
                let rows: Vec<Vec<String>> = nodes
                    .iter()
                    .map(|f| {
                        vec![
                            f.credit_facility_id.clone(),
                            f.public_id.clone(),
                            format!("{:?}", f.status),
                            scalar(&f.facility_amount),
                            format!("{:?}", f.collateralization_state),
                            f.activated_at.clone(),
                        ]
                    })
                    .collect();
                output::print_table(
                    &[
                        "ID",
                        "Public ID",
                        "Status",
                        "Amount",
                        "Collateral State",
                        "Activated",
                    ],
                    rows,
                );
                let pi = data.credit_facilities.page_info;
                if pi.has_next_page
                    && let Some(cursor) = pi.end_cursor
                {
                    println!("\nMore results available. Use --after {cursor}");
                }
            }
        }
        CreditFacilityAction::Get { id } => {
            let vars = credit_facility_get::Variables { id };
            let data = client.execute::<CreditFacilityGet>(vars).await?;
            match data.credit_facility {
                Some(f) => {
                    if json {
                        output::print_json(&f)?;
                    } else {
                        let amount = scalar(&f.facility_amount);
                        let rate = scalar(&f.credit_facility_terms.annual_rate);
                        output::print_kv(&[
                            ("Facility ID", &f.credit_facility_id),
                            ("Public ID", &f.public_id),
                            ("Status", &format!("{:?}", f.status)),
                            ("Amount", &amount),
                            (
                                "Collateral State",
                                &format!("{:?}", f.collateralization_state),
                            ),
                            ("Annual Rate", &rate),
                            (
                                "Duration",
                                &format!(
                                    "{} {:?}",
                                    f.credit_facility_terms.duration.units,
                                    f.credit_facility_terms.duration.period,
                                ),
                            ),
                            ("Matures At", &f.matures_at),
                            ("Activated At", &f.activated_at),
                        ]);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Credit facility not found");
                    }
                }
            }
        }
        CreditFacilityAction::ProposalGet { id } => {
            let vars = credit_facility_proposal_get::Variables { id };
            let data = client.execute::<CreditFacilityProposalGet>(vars).await?;
            match data.credit_facility_proposal {
                Some(p) => {
                    if json {
                        output::print_json(&p)?;
                    } else {
                        output::print_kv(&[
                            ("Proposal ID", &p.credit_facility_proposal_id),
                            ("Status", &format!("{:?}", p.status)),
                        ]);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Credit facility proposal not found");
                    }
                }
            }
        }
        CreditFacilityAction::ProposalConclude { id, approved } => {
            let vars = credit_facility_proposal_customer_approval_conclude::Variables {
                input: credit_facility_proposal_customer_approval_conclude::CreditFacilityProposalCustomerApprovalConcludeInput {
                    credit_facility_proposal_id: id,
                    approved,
                },
            };
            let data = client
                .execute::<CreditFacilityProposalCustomerApprovalConclude>(vars)
                .await?;
            let p = data
                .credit_facility_proposal_customer_approval_conclude
                .credit_facility_proposal;
            if json {
                output::print_json(&p)?;
            } else {
                output::print_kv(&[("Proposal ID", &p.credit_facility_proposal_id)]);
            }
        }
        CreditFacilityAction::PendingGet { id } => {
            let vars = pending_credit_facility_get::Variables { id };
            let data = client.execute::<PendingCreditFacilityGet>(vars).await?;
            match data.pending_credit_facility {
                Some(p) => {
                    if json {
                        output::print_json(&p)?;
                    } else {
                        let address = p
                            .wallet
                            .as_ref()
                            .map(|w| w.address.as_str())
                            .unwrap_or("N/A");
                        let btc_balance = scalar(&p.collateral.btc_balance);
                        output::print_kv(&[
                            ("Pending CF ID", &p.pending_credit_facility_id),
                            ("Collateral ID", &p.collateral_id),
                            ("Status", &format!("{:?}", p.status)),
                            ("Wallet Address", address),
                            ("BTC Balance", &btc_balance),
                        ]);
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Pending credit facility not found");
                    }
                }
            }
        }
        CreditFacilityAction::DisbursalInitiate {
            credit_facility_id,
            amount,
        } => {
            let vars = credit_facility_disbursal_initiate::Variables {
                input: credit_facility_disbursal_initiate::CreditFacilityDisbursalInitiateInput {
                    credit_facility_id,
                    amount: sval(amount),
                },
            };
            let data = client
                .execute::<CreditFacilityDisbursalInitiate>(vars)
                .await?;
            let d = data.credit_facility_disbursal_initiate.disbursal;
            if json {
                output::print_json(&d)?;
            } else {
                let amount = scalar(&d.amount);
                output::print_kv(&[
                    ("Disbursal ID", &d.disbursal_id),
                    ("Public ID", &d.public_id),
                    ("Amount", &amount),
                    ("Status", &format!("{:?}", d.status)),
                    ("Created", &d.created_at),
                ]);
            }
        }
        CreditFacilityAction::Find { id } => {
            let vars = credit_facility_find::Variables { id };
            let data = client.execute::<CreditFacilityFind>(vars).await?;
            match data.credit_facility {
                Some(f) => {
                    if json {
                        output::print_json(&f)?;
                    } else {
                        output::print_json(&f)?;
                    }
                }
                None => {
                    if json {
                        println!("null");
                    } else {
                        println!("Credit facility not found");
                    }
                }
            }
        }
        CreditFacilityAction::PartialPaymentRecord {
            credit_facility_id,
            amount,
        } => {
            let vars = credit_facility_partial_payment_record::Variables {
                input: credit_facility_partial_payment_record::CreditFacilityPartialPaymentRecordInput {
                    credit_facility_id,
                    amount: sval(amount),
                },
            };
            let data = client
                .execute::<CreditFacilityPartialPaymentRecord>(vars)
                .await?;
            let f = data.credit_facility_partial_payment_record.credit_facility;
            if json {
                output::print_json(&f)?;
            } else {
                output::print_json(&f)?;
            }
        }
    }
    Ok(())
}

fn parse_interest_interval(s: &str) -> Result<credit_facility_proposal_create::InterestInterval> {
    match s.to_uppercase().as_str() {
        "END_OF_MONTH" => Ok(credit_facility_proposal_create::InterestInterval::END_OF_MONTH),
        "END_OF_DAY" => Ok(credit_facility_proposal_create::InterestInterval::END_OF_DAY),
        other => anyhow::bail!("Unknown interest interval: {other}"),
    }
}

fn parse_disbursal_policy(s: &str) -> Result<credit_facility_proposal_create::DisbursalPolicy> {
    match s.to_uppercase().as_str() {
        "SINGLE_DISBURSAL" => {
            Ok(credit_facility_proposal_create::DisbursalPolicy::SINGLE_DISBURSAL)
        }
        "MULTIPLE_DISBURSAL" => {
            Ok(credit_facility_proposal_create::DisbursalPolicy::MULTIPLE_DISBURSAL)
        }
        other => anyhow::bail!("Unknown disbursal policy: {other}"),
    }
}
