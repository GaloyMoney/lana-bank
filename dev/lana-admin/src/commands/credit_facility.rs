use anyhow::{Result, bail};
use serde_json::json;
use tokio::time::{Duration, Instant, sleep};

use crate::cli::{
    CreditFacilityAction, DisbursalInitiateWaitFor, PendingCreditFacilityWaitFor,
    ProposalConcludeWaitFor,
};
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
            let requested_customer_id = customer_id.clone();
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
                output::print_json(&json!({
                    "creditFacilityProposalId": p.credit_facility_proposal_id,
                    "approvalProcessId": p.approval_process_id,
                    "customerId": requested_customer_id,
                    "status": format!("{:?}", p.status),
                    "facilityAmount": p.facility_amount,
                    "createdAt": p.created_at,
                    "terms": {
                        "annualRate": p.credit_facility_terms.annual_rate,
                        "oneTimeFeeRate": p.credit_facility_terms.one_time_fee_rate,
                        "disbursalPolicy": format!("{:?}", p.credit_facility_terms.disbursal_policy),
                        "duration": {
                            "period": format!("{:?}", p.credit_facility_terms.duration.period),
                            "units": p.credit_facility_terms.duration.units,
                        }
                    }
                }))?;
            } else {
                let amount = scalar(&p.facility_amount);
                let rate = scalar(&p.credit_facility_terms.annual_rate);
                output::print_kv(&[
                    ("Proposal ID", &p.credit_facility_proposal_id),
                    (
                        "Approval Process ID",
                        p.approval_process_id.as_deref().unwrap_or("N/A"),
                    ),
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
                        output::print_json(&credit_facility_get_json(&f))?;
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
                None => output::not_found("Credit facility", json),
            }
        }
        CreditFacilityAction::ProposalGet { id } => {
            let vars = credit_facility_proposal_get::Variables { id };
            let data = client.execute::<CreditFacilityProposalGet>(vars).await?;
            match data.credit_facility_proposal {
                Some(p) => {
                    if json {
                        output::print_json(&json!({
                            "creditFacilityProposalId": p.credit_facility_proposal_id,
                            "approvalProcessId": p.approval_process_id,
                            "status": format!("{:?}", p.status),
                        }))?;
                    } else {
                        output::print_kv(&[
                            ("Proposal ID", &p.credit_facility_proposal_id),
                            (
                                "Approval Process ID",
                                p.approval_process_id.as_deref().unwrap_or("N/A"),
                            ),
                            ("Status", &format!("{:?}", p.status)),
                        ]);
                    }
                }
                None => output::not_found("Credit facility proposal", json),
            }
        }
        CreditFacilityAction::ProposalConclude {
            id,
            approved,
            wait_for,
            wait_timeout_secs,
            wait_interval_ms,
        } => {
            if wait_for.is_some() && !approved {
                bail!("--wait-for is only valid when --approved true");
            }

            let vars = credit_facility_proposal_customer_approval_conclude::Variables {
                input: credit_facility_proposal_customer_approval_conclude::CreditFacilityProposalCustomerApprovalConcludeInput {
                    credit_facility_proposal_id: id.clone(),
                    approved,
                },
            };
            let data = client
                .execute::<CreditFacilityProposalCustomerApprovalConclude>(vars)
                .await?;
            let p = data
                .credit_facility_proposal_customer_approval_conclude
                .credit_facility_proposal;

            let pending = match wait_for {
                Some(ProposalConcludeWaitFor::PendingReady) => {
                    let (timeout, interval) = wait_durations(wait_timeout_secs, wait_interval_ms);
                    Some(
                        wait_for_pending_credit_facility_ready(client, &id, timeout, interval)
                            .await?,
                    )
                }
                None => None,
            };

            if json {
                output::print_json(&json!({
                    "creditFacilityProposalId": p.credit_facility_proposal_id,
                    "approvalProcessId": p.approval_process_id,
                    "status": format!("{:?}", p.status),
                    "approved": approved,
                    "pendingCreditFacilityId": pending.as_ref().map(|value| value.pending_credit_facility_id.clone()),
                    "creditFacilityId": pending.as_ref().map(|value| value.credit_facility_id.clone()),
                    "pendingApprovalProcessId": pending.as_ref().map(|value| value.approval_process_id.clone()),
                    "pendingStatus": pending.as_ref().map(|value| format!("{:?}", value.status)),
                    "collateralId": pending.as_ref().map(|value| value.collateral_id.clone()),
                }))?;
            } else {
                output::print_kv(&[
                    ("Proposal ID", &p.credit_facility_proposal_id),
                    (
                        "Approval Process ID",
                        p.approval_process_id.as_deref().unwrap_or("N/A"),
                    ),
                    ("Status", &format!("{:?}", p.status)),
                    ("Approved", if approved { "true" } else { "false" }),
                    (
                        "Pending CF ID",
                        pending
                            .as_ref()
                            .map(|value| value.pending_credit_facility_id.as_str())
                            .unwrap_or("N/A"),
                    ),
                    (
                        "Facility ID",
                        pending
                            .as_ref()
                            .map(|value| value.credit_facility_id.as_str())
                            .unwrap_or("N/A"),
                    ),
                ]);
            }
        }
        CreditFacilityAction::PendingGet {
            id,
            wait_for,
            wait_timeout_secs,
            wait_interval_ms,
        } => {
            let pending = match wait_for {
                Some(PendingCreditFacilityWaitFor::Completed) => {
                    let (timeout, interval) = wait_durations(wait_timeout_secs, wait_interval_ms);
                    wait_for_pending_credit_facility_completed(client, &id, timeout, interval)
                        .await?
                }
                None => {
                    let vars = pending_credit_facility_get::Variables { id };
                    let data = client.execute::<PendingCreditFacilityGet>(vars).await?;
                    match data.pending_credit_facility {
                        Some(p) => p,
                        None => output::not_found("Pending credit facility", json),
                    }
                }
            };

            if json {
                output::print_json(&pending_credit_facility_json(&pending))?;
            } else {
                let address = pending
                    .wallet
                    .as_ref()
                    .map(|w| w.address.as_str())
                    .unwrap_or("N/A");
                let btc_balance = scalar(&pending.collateral.btc_balance);
                output::print_kv(&[
                    ("Pending CF ID", &pending.pending_credit_facility_id),
                    ("Facility ID", &pending.credit_facility_id),
                    ("Approval Process ID", &pending.approval_process_id),
                    ("Collateral ID", &pending.collateral_id),
                    ("Status", &format!("{:?}", pending.status)),
                    (
                        "Collateral State",
                        &format!("{:?}", pending.collateralization_state),
                    ),
                    ("Wallet Address", address),
                    ("BTC Balance", &btc_balance),
                ]);
            }
        }
        CreditFacilityAction::DisbursalInitiate {
            credit_facility_id,
            amount,
            wait_for,
            wait_timeout_secs,
            wait_interval_ms,
        } => {
            let requested_credit_facility_id = credit_facility_id.clone();
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

            let final_status = match wait_for {
                Some(DisbursalInitiateWaitFor::Confirmed) => {
                    let (timeout, interval) = wait_durations(wait_timeout_secs, wait_interval_ms);
                    wait_for_disbursal_confirmed(
                        client,
                        &requested_credit_facility_id,
                        &d.credit_facility_disbursal_id,
                        timeout,
                        interval,
                    )
                    .await?
                }
                None => format!("{:?}", d.status),
            };

            if json {
                output::print_json(&json!({
                    "creditFacilityDisbursalId": d.credit_facility_disbursal_id,
                    "creditFacilityId": d.credit_facility.credit_facility_id,
                    "approvalProcessId": d.approval_process.approval_process_id,
                    "amount": d.amount,
                    "status": final_status,
                    "createdAt": d.created_at,
                    "publicId": d.public_id,
                }))?;
            } else {
                let amount = scalar(&d.amount);
                output::print_kv(&[
                    ("Disbursal ID", &d.credit_facility_disbursal_id),
                    ("Facility ID", &d.credit_facility.credit_facility_id),
                    (
                        "Approval Process ID",
                        &d.approval_process.approval_process_id,
                    ),
                    ("Public ID", &d.public_id),
                    ("Amount", &amount),
                    ("Status", &final_status),
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
                        output::print_json(&credit_facility_find_json(&f))?;
                    } else {
                        output::print_kv(&[
                            ("Facility ID", &f.credit_facility_id),
                            ("Collateral ID", &f.collateral_id),
                            ("Status", &format!("{:?}", f.status)),
                            (
                                "Collateral State",
                                &format!("{:?}", f.collateralization_state),
                            ),
                        ]);
                    }
                }
                None => output::not_found("Credit facility", json),
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
                output::print_json(&json!({
                    "creditFacilityId": f.credit_facility_id,
                    "facilityAmount": f.facility_amount,
                    "balance": {
                        "facilityRemainingUsdBalance": f.balance.facility_remaining.usd_balance,
                        "outstandingUsdBalance": f.balance.outstanding.usd_balance,
                        "collateralBtcBalance": f.balance.collateral.btc_balance,
                        "disbursedTotalUsdBalance": f.balance.disbursed.total.usd_balance,
                        "disbursedOutstandingUsdBalance": f.balance.disbursed.outstanding.usd_balance,
                        "interestTotalUsdBalance": f.balance.interest.total.usd_balance,
                        "interestOutstandingUsdBalance": f.balance.interest.outstanding.usd_balance,
                        "paymentsUnappliedUsdBalance": f.balance.payments_unapplied.usd_balance,
                    }
                }))?;
            } else {
                output::print_kv(&[
                    ("Facility ID", &f.credit_facility_id),
                    (
                        "Outstanding USD",
                        &scalar(&f.balance.outstanding.usd_balance),
                    ),
                    (
                        "Payments Unapplied USD",
                        &scalar(&f.balance.payments_unapplied.usd_balance),
                    ),
                ]);
            }
        }
    }
    Ok(())
}

fn credit_facility_get_json(
    f: &credit_facility_get::CreditFacilityGetCreditFacility,
) -> serde_json::Value {
    json!({
        "creditFacilityId": f.credit_facility_id,
        "publicId": f.public_id,
        "status": format!("{:?}", f.status),
        "facilityAmount": f.facility_amount,
        "collateralizationState": format!("{:?}", f.collateralization_state),
        "maturesAt": f.matures_at,
        "activatedAt": f.activated_at,
        "terms": {
            "annualRate": f.credit_facility_terms.annual_rate,
            "oneTimeFeeRate": f.credit_facility_terms.one_time_fee_rate,
            "disbursalPolicy": format!("{:?}", f.credit_facility_terms.disbursal_policy),
            "duration": {
                "period": format!("{:?}", f.credit_facility_terms.duration.period),
                "units": f.credit_facility_terms.duration.units,
            }
        }
    })
}

fn credit_facility_find_json(
    f: &credit_facility_find::CreditFacilityFindCreditFacility,
) -> serde_json::Value {
    json!({
        "creditFacilityId": f.credit_facility_id,
        "collateralId": f.collateral_id,
        "status": format!("{:?}", f.status),
        "maturesAt": f.matures_at,
        "collateralizationState": format!("{:?}", f.collateralization_state),
        "facilityAmount": f.facility_amount,
        "disbursals": f.disbursals.iter().map(|disbursal| {
            json!({
                "creditFacilityDisbursalId": disbursal.credit_facility_disbursal_id,
                "status": format!("{:?}", disbursal.status),
            })
        }).collect::<Vec<_>>(),
        "history": f.history,
        "balance": {
            "facilityRemainingUsdBalance": f.balance.facility_remaining.usd_balance,
            "outstandingUsdBalance": f.balance.outstanding.usd_balance,
            "collateralBtcBalance": f.balance.collateral.btc_balance,
            "disbursedTotalUsdBalance": f.balance.disbursed.total.usd_balance,
            "disbursedOutstandingUsdBalance": f.balance.disbursed.outstanding.usd_balance,
            "interestTotalUsdBalance": f.balance.interest.total.usd_balance,
            "interestOutstandingUsdBalance": f.balance.interest.outstanding.usd_balance,
            "paymentsUnappliedUsdBalance": f.balance.payments_unapplied.usd_balance,
        },
        "liquidationIds": f.liquidations.iter().map(|liquidation| liquidation.liquidation_id.clone()).collect::<Vec<_>>(),
    })
}

fn pending_credit_facility_json(
    p: &pending_credit_facility_get::PendingCreditFacilityGetPendingCreditFacility,
) -> serde_json::Value {
    json!({
        "pendingCreditFacilityId": p.pending_credit_facility_id,
        "creditFacilityId": p.credit_facility_id,
        "approvalProcessId": p.approval_process_id,
        "collateralId": p.collateral_id,
        "status": format!("{:?}", p.status),
        "createdAt": p.created_at,
        "facilityAmount": p.facility_amount,
        "collateralizationState": format!("{:?}", p.collateralization_state),
        "walletAddress": p.wallet.as_ref().map(|wallet| wallet.address.clone()),
        "btcBalance": p.collateral.btc_balance,
    })
}

async fn wait_for_pending_credit_facility_ready(
    client: &mut GraphQLClient,
    proposal_id: &str,
    timeout: Duration,
    interval: Duration,
) -> Result<pending_credit_facility_get::PendingCreditFacilityGetPendingCreditFacility> {
    let deadline = Instant::now() + timeout;
    loop {
        let vars = pending_credit_facility_get::Variables {
            id: proposal_id.to_string(),
        };
        let data = client.execute::<PendingCreditFacilityGet>(vars).await?;
        if let Some(pending) = data.pending_credit_facility {
            return Ok(pending);
        }
        if Instant::now() >= deadline {
            bail!("Timed out waiting for pending credit facility to become queryable");
        }
        sleep(interval).await;
    }
}

async fn wait_for_pending_credit_facility_completed(
    client: &mut GraphQLClient,
    pending_id: &str,
    timeout: Duration,
    interval: Duration,
) -> Result<pending_credit_facility_get::PendingCreditFacilityGetPendingCreditFacility> {
    let deadline = Instant::now() + timeout;
    loop {
        let vars = pending_credit_facility_get::Variables {
            id: pending_id.to_string(),
        };
        let data = client.execute::<PendingCreditFacilityGet>(vars).await?;
        if let Some(pending) = data.pending_credit_facility
            && format!("{:?}", pending.status) == "COMPLETED"
        {
            return Ok(pending);
        }
        if Instant::now() >= deadline {
            bail!("Timed out waiting for pending credit facility to reach COMPLETED");
        }
        sleep(interval).await;
    }
}

async fn wait_for_disbursal_confirmed(
    client: &mut GraphQLClient,
    credit_facility_id: &str,
    disbursal_id: &str,
    timeout: Duration,
    interval: Duration,
) -> Result<String> {
    let deadline = Instant::now() + timeout;
    loop {
        let vars = credit_facility_find::Variables {
            id: credit_facility_id.to_string(),
        };
        let data = client.execute::<CreditFacilityFind>(vars).await?;
        if let Some(facility) = data.credit_facility
            && let Some(disbursal) = facility
                .disbursals
                .iter()
                .find(|disbursal| disbursal.credit_facility_disbursal_id == disbursal_id)
        {
            let status = format!("{:?}", disbursal.status);
            if status == "CONFIRMED" {
                return Ok(status);
            }
        }
        if Instant::now() >= deadline {
            bail!("Timed out waiting for disbursal to reach CONFIRMED");
        }
        sleep(interval).await;
    }
}

fn wait_durations(timeout_secs: u64, interval_ms: u64) -> (Duration, Duration) {
    (
        Duration::from_secs(timeout_secs.max(1)),
        Duration::from_millis(interval_ms.max(1)),
    )
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
