use std::sync::Arc;

use tokio::sync::Mutex;

use crate::client::GraphQLClient;
use crate::graphql::*;
use crate::output::scalar;

use super::app::{Action, DetailResult, Domain, ListResult};

pub async fn fetch_list(
    client: &Arc<Mutex<GraphQLClient>>,
    domain: Domain,
    first: i64,
    after: Option<String>,
) -> anyhow::Result<ListResult> {
    let mut client = client.lock().await;
    match domain {
        Domain::Prospects => {
            let vars = prospects_list::Variables { first, after };
            let data = client.execute::<ProspectsList>(vars).await?;
            let pi = data.prospects.page_info;
            let nodes = data.prospects.nodes;
            Ok(ListResult {
                headers: vec![
                    "ID",
                    "Public ID",
                    "Email",
                    "Type",
                    "Stage",
                    "Status",
                    "Created",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                ids: nodes.iter().map(|p| p.prospect_id.clone()).collect(),
                rows: nodes
                    .iter()
                    .map(|p| {
                        vec![
                            p.prospect_id.clone(),
                            p.public_id.clone(),
                            p.email.clone(),
                            format!("{:?}", p.customer_type),
                            format!("{:?}", p.stage),
                            format!("{:?}", p.status),
                            p.created_at.clone(),
                        ]
                    })
                    .collect(),
                has_next_page: pi.has_next_page,
                end_cursor: pi.end_cursor,
            })
        }
        Domain::Customers => {
            let vars = customers_list::Variables { first, after };
            let data = client.execute::<CustomersList>(vars).await?;
            let pi = data.customers.page_info;
            let nodes = data.customers.nodes;
            Ok(ListResult {
                headers: vec![
                    "ID",
                    "Public ID",
                    "Email",
                    "Type",
                    "Activity",
                    "Level",
                    "Created",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                ids: nodes.iter().map(|c| c.customer_id.clone()).collect(),
                rows: nodes
                    .iter()
                    .map(|c| {
                        vec![
                            c.customer_id.clone(),
                            c.public_id.clone(),
                            c.email.clone(),
                            format!("{:?}", c.customer_type),
                            format!("{:?}", c.activity),
                            format!("{:?}", c.level),
                            c.created_at.clone(),
                        ]
                    })
                    .collect(),
                has_next_page: pi.has_next_page,
                end_cursor: pi.end_cursor,
            })
        }
        Domain::DepositAccounts => {
            let vars = deposit_accounts_list::Variables { first, after };
            let data = client.execute::<DepositAccountsList>(vars).await?;
            let pi = data.deposit_accounts.page_info;
            let nodes = data.deposit_accounts.nodes;
            Ok(ListResult {
                headers: vec![
                    "Account ID",
                    "Customer ID",
                    "Public ID",
                    "Status",
                    "Created",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                ids: nodes.iter().map(|a| a.deposit_account_id.clone()).collect(),
                rows: nodes
                    .iter()
                    .map(|a| {
                        vec![
                            a.deposit_account_id.clone(),
                            a.customer_id.clone(),
                            a.public_id.clone(),
                            format!("{:?}", a.status),
                            a.created_at.clone(),
                        ]
                    })
                    .collect(),
                has_next_page: pi.has_next_page,
                end_cursor: pi.end_cursor,
            })
        }
        Domain::TermsTemplates => {
            let vars = terms_templates_list::Variables;
            let data = client.execute::<TermsTemplatesList>(vars).await?;
            let templates = data.terms_templates;
            Ok(ListResult {
                headers: vec![
                    "ID",
                    "Name",
                    "Annual Rate",
                    "Disbursal",
                    "Duration",
                    "Created",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                ids: templates.iter().map(|t| t.terms_id.clone()).collect(),
                rows: templates
                    .iter()
                    .map(|t| {
                        vec![
                            t.terms_id.clone(),
                            t.name.clone(),
                            scalar(&t.values.annual_rate),
                            format!("{:?}", t.values.disbursal_policy),
                            format!("{} {:?}", t.values.duration.units, t.values.duration.period),
                            t.created_at.clone(),
                        ]
                    })
                    .collect(),
                has_next_page: false,
                end_cursor: None,
            })
        }
        Domain::CreditFacilities => {
            let vars = credit_facilities_list::Variables { first, after };
            let data = client.execute::<CreditFacilitiesList>(vars).await?;
            let pi = data.credit_facilities.page_info;
            let nodes = data.credit_facilities.nodes;
            Ok(ListResult {
                headers: vec![
                    "ID",
                    "Public ID",
                    "Status",
                    "Amount",
                    "Collateral State",
                    "Activated",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                ids: nodes.iter().map(|f| f.credit_facility_id.clone()).collect(),
                rows: nodes
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
                    .collect(),
                has_next_page: pi.has_next_page,
                end_cursor: pi.end_cursor,
            })
        }
        Domain::CreditFacilityProposals => {
            let vars = credit_facility_proposals_list::Variables { first, after };
            let data = client.execute::<CreditFacilityProposalsList>(vars).await?;
            let pi = data.credit_facility_proposals.page_info;
            let nodes = data.credit_facility_proposals.nodes;
            Ok(ListResult {
                headers: vec!["Proposal ID", "Status", "Amount", "Created"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                ids: nodes
                    .iter()
                    .map(|p| p.credit_facility_proposal_id.clone())
                    .collect(),
                rows: nodes
                    .iter()
                    .map(|p| {
                        vec![
                            p.credit_facility_proposal_id.clone(),
                            format!("{:?}", p.status),
                            scalar(&p.facility_amount),
                            p.created_at.clone(),
                        ]
                    })
                    .collect(),
                has_next_page: pi.has_next_page,
                end_cursor: pi.end_cursor,
            })
        }
        Domain::ApprovalProcesses => {
            let vars = approval_processes_list::Variables { first, after };
            let data = client.execute::<ApprovalProcessesList>(vars).await?;
            let pi = data.approval_processes.page_info;
            let nodes = data.approval_processes.nodes;
            Ok(ListResult {
                headers: vec!["Process ID", "Type", "Status", "Created"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                ids: nodes
                    .iter()
                    .map(|ap| ap.approval_process_id.clone())
                    .collect(),
                rows: nodes
                    .iter()
                    .map(|ap| {
                        vec![
                            ap.approval_process_id.clone(),
                            format!("{:?}", ap.approval_process_type),
                            format!("{:?}", ap.status),
                            ap.created_at.clone(),
                        ]
                    })
                    .collect(),
                has_next_page: pi.has_next_page,
                end_cursor: pi.end_cursor,
            })
        }
    }
}

pub async fn fetch_detail(
    client: &Arc<Mutex<GraphQLClient>>,
    domain: Domain,
    id: &str,
) -> anyhow::Result<DetailResult> {
    let mut client = client.lock().await;
    match domain {
        Domain::Prospects => {
            let vars = prospect_get::Variables { id: id.to_string() };
            let data = client.execute::<ProspectGet>(vars).await?;
            match data.prospect {
                Some(p) => Ok(DetailResult {
                    pairs: vec![
                        ("Prospect ID".into(), p.prospect_id),
                        ("Public ID".into(), p.public_id),
                        ("Email".into(), p.email),
                        ("Telegram".into(), p.telegram_handle),
                        ("Type".into(), format!("{:?}", p.customer_type)),
                        ("Stage".into(), format!("{:?}", p.stage)),
                        ("Status".into(), format!("{:?}", p.status)),
                        ("KYC Status".into(), format!("{:?}", p.kyc_status)),
                        ("Level".into(), format!("{:?}", p.level)),
                        ("Created".into(), p.created_at),
                        (
                            "Applicant ID".into(),
                            p.applicant_id.unwrap_or_else(|| "N/A".into()),
                        ),
                    ],
                    actions: vec![Action::Convert, Action::Close],
                }),
                None => anyhow::bail!("Prospect not found"),
            }
        }
        Domain::Customers => {
            let vars = customer_get::Variables { id: id.to_string() };
            let data = client.execute::<CustomerGet>(vars).await?;
            match data.customer {
                Some(c) => Ok(DetailResult {
                    pairs: vec![
                        ("Customer ID".into(), c.customer_id),
                        ("Public ID".into(), c.public_id),
                        ("Email".into(), c.email),
                        ("Telegram".into(), c.telegram_handle),
                        ("Type".into(), format!("{:?}", c.customer_type)),
                        ("Activity".into(), format!("{:?}", c.activity)),
                        ("Level".into(), format!("{:?}", c.level)),
                        ("KYC".into(), format!("{:?}", c.kyc_verification)),
                        ("Created".into(), c.created_at),
                        ("Applicant ID".into(), c.applicant_id),
                    ],
                    actions: vec![],
                }),
                None => anyhow::bail!("Customer not found"),
            }
        }
        Domain::DepositAccounts => {
            let vars = deposit_account_get::Variables { id: id.to_string() };
            let data = client.execute::<DepositAccountGet>(vars).await?;
            match data.deposit_account {
                Some(a) => Ok(DetailResult {
                    pairs: vec![
                        ("Account ID".into(), a.deposit_account_id),
                        ("Customer ID".into(), a.customer_id),
                        ("Public ID".into(), a.public_id),
                        ("Status".into(), format!("{:?}", a.status)),
                        ("Created".into(), a.created_at),
                        ("Balance (settled)".into(), scalar(&a.balance.settled)),
                        ("Balance (pending)".into(), scalar(&a.balance.pending)),
                    ],
                    actions: vec![],
                }),
                None => anyhow::bail!("Deposit account not found"),
            }
        }
        Domain::CreditFacilities => {
            let vars = credit_facility_get::Variables { id: id.to_string() };
            let data = client.execute::<CreditFacilityGet>(vars).await?;
            match data.credit_facility {
                Some(f) => Ok(DetailResult {
                    pairs: vec![
                        ("Facility ID".into(), f.credit_facility_id),
                        ("Public ID".into(), f.public_id),
                        ("Status".into(), format!("{:?}", f.status)),
                        ("Amount".into(), scalar(&f.facility_amount)),
                        (
                            "Collateral State".into(),
                            format!("{:?}", f.collateralization_state),
                        ),
                        (
                            "Annual Rate".into(),
                            scalar(&f.credit_facility_terms.annual_rate),
                        ),
                        (
                            "Duration".into(),
                            format!(
                                "{} {:?}",
                                f.credit_facility_terms.duration.units,
                                f.credit_facility_terms.duration.period,
                            ),
                        ),
                        ("Matures At".into(), f.matures_at),
                        ("Activated At".into(), f.activated_at),
                    ],
                    actions: vec![],
                }),
                None => anyhow::bail!("Credit facility not found"),
            }
        }
        Domain::ApprovalProcesses => {
            let vars = approval_process_get::Variables { id: id.to_string() };
            let data = client.execute::<ApprovalProcessGet>(vars).await?;
            match data.approval_process {
                Some(ap) => {
                    let actions = if ap.user_can_submit_decision {
                        vec![Action::Approve, Action::Deny]
                    } else {
                        vec![]
                    };
                    Ok(DetailResult {
                        pairs: vec![
                            ("Process ID".into(), ap.approval_process_id),
                            ("Type".into(), format!("{:?}", ap.approval_process_type)),
                            ("Status".into(), format!("{:?}", ap.status)),
                            (
                                "Denied Reason".into(),
                                ap.denied_reason.unwrap_or_else(|| "N/A".into()),
                            ),
                            (
                                "Can Submit Decision".into(),
                                ap.user_can_submit_decision.to_string(),
                            ),
                            ("Created".into(), ap.created_at),
                        ],
                        actions,
                    })
                }
                None => anyhow::bail!("Approval process not found"),
            }
        }
        Domain::TermsTemplates | Domain::CreditFacilityProposals => {
            anyhow::bail!("Detail view not available for this domain")
        }
    }
}

pub async fn execute_action(
    client: &Arc<Mutex<GraphQLClient>>,
    domain: Domain,
    entity_id: &str,
    action: Action,
    input: Option<String>,
) -> anyhow::Result<String> {
    let mut client = client.lock().await;
    match (domain, action) {
        (Domain::Prospects, Action::Convert) => {
            let vars = prospect_convert::Variables {
                input: prospect_convert::ProspectConvertInput {
                    prospect_id: entity_id.to_string(),
                },
            };
            let data = client.execute::<ProspectConvert>(vars).await?;
            let c = data.prospect_convert.customer;
            Ok(format!("Prospect converted to customer {}", c.customer_id))
        }
        (Domain::Prospects, Action::Close) => {
            let vars = prospect_close::Variables {
                input: prospect_close::ProspectCloseInput {
                    prospect_id: entity_id.to_string(),
                },
            };
            client.execute::<ProspectClose>(vars).await?;
            Ok("Prospect closed".into())
        }
        (Domain::ApprovalProcesses, Action::Approve) => {
            let vars = approval_process_approve::Variables {
                input: approval_process_approve::ApprovalProcessApproveInput {
                    process_id: entity_id.to_string(),
                },
            };
            client.execute::<ApprovalProcessApprove>(vars).await?;
            Ok("Approval submitted".into())
        }
        (Domain::ApprovalProcesses, Action::Deny) => {
            let reason = input.unwrap_or_default();
            let vars = approval_process_deny::Variables {
                input: approval_process_deny::ApprovalProcessDenyInput {
                    process_id: entity_id.to_string(),
                },
                reason,
            };
            client.execute::<ApprovalProcessDeny>(vars).await?;
            Ok("Denial submitted".into())
        }
        _ => anyhow::bail!("Action not supported for this domain"),
    }
}
