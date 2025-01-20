use std::collections::HashMap;

use crate::{path::*, ChartId};

use super::ChartEvent;

pub struct ChartTreeCategory {
    pub name: String,
    pub encoded_path: String,
    pub children: Vec<ChartTreeControlAccount>,
}

struct ControlAccountAdded {
    name: String,
    path: ControlAccountPath,
}

pub struct ChartTreeControlAccount {
    pub name: String,
    pub encoded_path: String,
    pub children: Vec<ChartTreeControlSubAccount>,
}

pub struct ChartTreeControlSubAccount {
    pub name: String,
    pub encoded_path: String,
}

pub struct ChartTree {
    pub id: ChartId,
    pub name: String,
    pub assets: ChartTreeCategory,
    pub liabilities: ChartTreeCategory,
    pub equity: ChartTreeCategory,
    pub revenues: ChartTreeCategory,
    pub expenses: ChartTreeCategory,
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
                    encoded_path: path.to_string(),
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
                encoded_path: account.path.to_string(),
                children: control_sub_accounts_by_parent
                    .remove(&account.path.to_string())
                    .unwrap_or_default(),
            });
    }

    ChartTree {
        id: id.expect("Chart must be initialized"),
        name: name.expect("Chart must be initialized"),
        assets: ChartTreeCategory {
            name: "Assets".to_string(),
            encoded_path: ChartCategory::Assets.to_string(),
            children: control_accounts_by_category
                .remove(&ChartCategory::Assets)
                .unwrap_or_default(),
        },
        liabilities: ChartTreeCategory {
            name: "Liabilities".to_string(),
            encoded_path: ChartCategory::Liabilities.to_string(),
            children: control_accounts_by_category
                .remove(&ChartCategory::Liabilities)
                .unwrap_or_default(),
        },
        equity: ChartTreeCategory {
            name: "Equity".to_string(),
            encoded_path: ChartCategory::Equity.to_string(),
            children: control_accounts_by_category
                .remove(&ChartCategory::Equity)
                .unwrap_or_default(),
        },
        revenues: ChartTreeCategory {
            name: "Revenues".to_string(),
            encoded_path: ChartCategory::Revenues.to_string(),
            children: control_accounts_by_category
                .remove(&ChartCategory::Revenues)
                .unwrap_or_default(),
        },
        expenses: ChartTreeCategory {
            name: "Expenses".to_string(),
            encoded_path: ChartCategory::Expenses.to_string(),
            children: control_accounts_by_category
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
                    LedgerAccountSetId::new(),
                    ChartCategory::Assets,
                    "Loans Receivable".to_string(),
                    "loans-receivable".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_control_sub_account(
                    LedgerAccountSetId::new(),
                    control_account.path,
                    "Fixed Loans Receivable".to_string(),
                    "fixed-loans-receivable".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(
            chart.chart().assets.children[0].children[0].encoded_path,
            "10101".to_string()
        );

        {
            let control_account = chart
                .create_control_account(
                    LedgerAccountSetId::new(),
                    ChartCategory::Liabilities,
                    "User Checking".to_string(),
                    "user-checking".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_control_sub_account(
                    LedgerAccountSetId::new(),
                    control_account.path,
                    "User Checking".to_string(),
                    "sub-user-checking".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(
            chart.chart().liabilities.children[0].children[0].encoded_path,
            "20101".to_string()
        );

        {
            let control_account = chart
                .create_control_account(
                    LedgerAccountSetId::new(),
                    ChartCategory::Equity,
                    "Shareholder Equity".to_string(),
                    "shareholder-equity".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
            chart
                .create_control_sub_account(
                    LedgerAccountSetId::new(),
                    control_account.path,
                    "Shareholder Equity".to_string(),
                    "sub-shareholder-equity".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(
            chart.chart().equity.children[0].children[0].encoded_path,
            "30101"
        );

        {
            chart
                .create_control_account(
                    LedgerAccountSetId::new(),
                    ChartCategory::Revenues,
                    "Interest Revenue".to_string(),
                    "interest-revenue".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(chart.chart().revenues.children[0].encoded_path, "40100");
        assert_eq!(chart.chart().revenues.children[0].children.len(), 0);

        assert_eq!(chart.chart().expenses.children.len(), 0);
    }
}
