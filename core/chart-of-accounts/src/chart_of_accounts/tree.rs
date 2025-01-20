use std::collections::HashMap;

use crate::{path::*, ChartId};

use super::ChartEvent;

#[derive(Clone)]
pub struct ChartTree {
    pub id: ChartId,
    pub name: String,
    pub assets: ChartTreeCategory,
    pub liabilities: ChartTreeCategory,
    pub equity: ChartTreeCategory,
    pub revenues: ChartTreeCategory,
    pub expenses: ChartTreeCategory,
}

#[derive(Clone)]
pub struct ChartTreeCategory {
    pub name: String,
    pub account_code: String,
    pub control_accounts: Vec<ChartTreeControlAccount>,
}

struct ControlAccountAdded {
    name: String,
    path: ControlAccountPath,
}

#[derive(Clone)]
pub struct ChartTreeControlAccount {
    pub name: String,
    pub account_code: String,
    pub control_sub_accounts: Vec<ChartTreeControlSubAccount>,
}

#[derive(Clone)]
pub struct ChartTreeControlSubAccount {
    pub name: String,
    pub account_code: String,
}

pub(super) fn project<'a>(events: impl DoubleEndedIterator<Item = &'a ChartEvent>) -> ChartTree {
    let mut id: Option<ChartId> = None;
    let mut name: Option<String> = None;
    let mut control_accounts_added: Vec<ControlAccountAdded> = vec![];
    let mut control_sub_accounts_by_parent: HashMap<String, Vec<ChartTreeControlSubAccount>> =
        HashMap::new();

    for event in events {
        match event {
            ChartEvent::Initialized {
                id: chart_id,
                name: chart_name,
                ..
            } => {
                id = Some(*chart_id);
                name = Some(chart_name.to_string());
            }
            ChartEvent::ControlAccountAdded { path, name, .. } => {
                control_accounts_added.push(ControlAccountAdded {
                    name: name.to_string(),
                    path: *path,
                })
            }
            ChartEvent::ControlSubAccountAdded { path, name, .. } => control_sub_accounts_by_parent
                .entry(path.control_account().to_string())
                .or_default()
                .push(ChartTreeControlSubAccount {
                    name: name.to_string(),
                    account_code: path.to_string(),
                }),
        }
    }

    let mut control_accounts_by_category: HashMap<ChartCategory, Vec<ChartTreeControlAccount>> =
        HashMap::new();
    for account in control_accounts_added {
        control_accounts_by_category
            .entry(account.path.category)
            .or_default()
            .push(ChartTreeControlAccount {
                name: account.name,
                account_code: account.path.to_string(),
                control_sub_accounts: control_sub_accounts_by_parent
                    .remove(&account.path.to_string())
                    .unwrap_or_default(),
            });
    }

    ChartTree {
        id: id.expect("Chart must be initialized"),
        name: name.expect("Chart must be initialized"),
        assets: ChartTreeCategory {
            name: "Assets".to_string(),
            account_code: ChartCategory::Assets.to_string(),
            control_accounts: control_accounts_by_category
                .remove(&ChartCategory::Assets)
                .unwrap_or_default(),
        },
        liabilities: ChartTreeCategory {
            name: "Liabilities".to_string(),
            account_code: ChartCategory::Liabilities.to_string(),
            control_accounts: control_accounts_by_category
                .remove(&ChartCategory::Liabilities)
                .unwrap_or_default(),
        },
        equity: ChartTreeCategory {
            name: "Equity".to_string(),
            account_code: ChartCategory::Equity.to_string(),
            control_accounts: control_accounts_by_category
                .remove(&ChartCategory::Equity)
                .unwrap_or_default(),
        },
        revenues: ChartTreeCategory {
            name: "Revenues".to_string(),
            account_code: ChartCategory::Revenues.to_string(),
            control_accounts: control_accounts_by_category
                .remove(&ChartCategory::Revenues)
                .unwrap_or_default(),
        },
        expenses: ChartTreeCategory {
            name: "Expenses".to_string(),
            account_code: ChartCategory::Expenses.to_string(),
            control_accounts: control_accounts_by_category
                .remove(&ChartCategory::Expenses)
                .unwrap_or_default(),
        },
    }
}

#[cfg(test)]
mod tests {
    use es_entity::*;

    use crate::{path::ChartCategory, Chart, LedgerAccountSetId, NewChart};

    use super::*;

    use audit::{AuditEntryId, AuditInfo};

    fn dummy_audit_info() -> AuditInfo {
        AuditInfo {
            audit_entry_id: AuditEntryId::from(1),
            sub: "sub".to_string(),
        }
    }

    fn init_chart_of_events() -> Chart {
        let id = ChartId::new();
        let audit_info = dummy_audit_info();

        let new_chart = NewChart::builder()
            .id(id)
            .name("Test Chart".to_string())
            .reference("ref-01".to_string())
            .audit_info(audit_info)
            .build()
            .unwrap();

        let events = new_chart.into_events();
        Chart::try_from_events(events).unwrap()
    }

    #[test]
    fn test_project_chart_structure() {
        let mut chart = init_chart_of_events();

        {
            let control_account = chart
                .create_control_account(
                    ChartCategory::Assets,
                    "Loans Receivable".to_string(),
                    "loans-receivable".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_control_sub_account(
                    LedgerAccountSetId::new(),
                    control_account,
                    "Fixed Loans Receivable".to_string(),
                    "fixed-loans-receivable".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(
            chart.chart().assets.control_accounts[0].control_sub_accounts[0].account_code,
            "10101".to_string()
        );

        {
            let control_account = chart
                .create_control_account(
                    ChartCategory::Liabilities,
                    "User Checking".to_string(),
                    "user-checking".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_control_sub_account(
                    LedgerAccountSetId::new(),
                    control_account,
                    "User Checking".to_string(),
                    "sub-user-checking".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(
            chart.chart().liabilities.control_accounts[0].control_sub_accounts[0].account_code,
            "20101".to_string()
        );

        {
            let control_account = chart
                .create_control_account(
                    ChartCategory::Equity,
                    "Shareholder Equity".to_string(),
                    "shareholder-equity".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_control_sub_account(
                    LedgerAccountSetId::new(),
                    control_account,
                    "Shareholder Equity".to_string(),
                    "sub-shareholder-equity".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(
            chart.chart().equity.control_accounts[0].control_sub_accounts[0].account_code,
            "30101"
        );

        {
            chart
                .create_control_account(
                    ChartCategory::Revenues,
                    "Interest Revenue".to_string(),
                    "interest-revenue".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(
            chart.chart().revenues.control_accounts[0].account_code,
            "40100"
        );
        assert_eq!(
            chart.chart().revenues.control_accounts[0]
                .control_sub_accounts
                .len(),
            0
        );

        assert_eq!(chart.chart().expenses.control_accounts.len(), 0);
    }
}
