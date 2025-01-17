use std::collections::HashMap;

use crate::{path::*, ChartId};

use super::ChartEvent;

pub struct CategoryProjection {
    pub name: String,
    pub encoded_path: String,
    pub children: Vec<ControlAccountProjection>,
}

struct ControlAccountAdded {
    name: String,
    path: ControlAccountPath,
}

pub struct ControlAccountProjection {
    pub name: String,
    pub encoded_path: String,
    pub children: Vec<ControlSubAccountProjection>,
}

pub struct ControlSubAccountProjection {
    pub name: String,
    pub encoded_path: String,
}

pub struct ChartOfAccountsProjection {
    pub id: ChartId,
    pub name: String,
    pub assets: CategoryProjection,
    pub liabilities: CategoryProjection,
    pub equity: CategoryProjection,
    pub revenues: CategoryProjection,
    pub expenses: CategoryProjection,
}

pub(super) fn project<'a>(
    events: impl DoubleEndedIterator<Item = &'a ChartEvent>,
) -> ChartOfAccountsProjection {
    let mut id: Option<ChartId> = None;
    let mut name: Option<String> = None;
    let mut control_accounts_added: Vec<ControlAccountAdded> = vec![];
    let mut control_sub_accounts_by_parent: HashMap<String, Vec<ControlSubAccountProjection>> =
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
                .push(ControlSubAccountProjection {
                    name: name.to_string(),
                    encoded_path: path.to_string(),
                }),
        }
    }

    let mut control_accounts_by_category: HashMap<ChartCategory, Vec<ControlAccountProjection>> =
        HashMap::new();
    for account in control_accounts_added {
        control_accounts_by_category
            .entry(account.path.category)
            .or_default()
            .push(ControlAccountProjection {
                name: account.name,
                encoded_path: account.path.to_string(),
                children: control_sub_accounts_by_parent
                    .remove(&account.path.to_string())
                    .unwrap_or_default(),
            });
    }

    ChartOfAccountsProjection {
        id: id.expect("Chart must be initialized"),
        name: name.expect("Chart must be initialized"),
        assets: CategoryProjection {
            name: "Assets".to_string(),
            encoded_path: ChartCategory::Assets.to_string(),
            children: control_accounts_by_category
                .remove(&ChartCategory::Assets)
                .unwrap_or_default(),
        },
        liabilities: CategoryProjection {
            name: "Liabilities".to_string(),
            encoded_path: ChartCategory::Liabilities.to_string(),
            children: control_accounts_by_category
                .remove(&ChartCategory::Liabilities)
                .unwrap_or_default(),
        },
        equity: CategoryProjection {
            name: "Equity".to_string(),
            encoded_path: ChartCategory::Equity.to_string(),
            children: control_accounts_by_category
                .remove(&ChartCategory::Equity)
                .unwrap_or_default(),
        },
        revenues: CategoryProjection {
            name: "Revenues".to_string(),
            encoded_path: ChartCategory::Revenues.to_string(),
            children: control_accounts_by_category
                .remove(&ChartCategory::Revenues)
                .unwrap_or_default(),
        },
        expenses: CategoryProjection {
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

    use crate::{path::ChartCategory, Chart, ChartCreationDetails, LedgerAccountId, NewChart};

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
                    control_account,
                    "Fixed Loans Receivable".to_string(),
                    "fixed-loans-receivable".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(
            chart.chart().assets.children[0].children[0].encoded_path,
            "10101000".to_string()
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
                    control_account,
                    "User Checking".to_string(),
                    "sub-user-checking".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(
            chart.chart().liabilities.children[0].children[0].encoded_path,
            "20101000".to_string()
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
                    control_account,
                    "Shareholder Equity".to_string(),
                    "sub-shareholder-equity".to_string(),
                    dummy_audit_info(),
                )
                .unwrap();
        }
        assert_eq!(
            chart.chart().equity.children[0].children[0].encoded_path,
            "30101000"
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
        assert_eq!(chart.chart().revenues.children[0].encoded_path, "40100000");
        assert_eq!(chart.chart().revenues.children[0].children.len(), 0);

        assert_eq!(chart.chart().expenses.children.len(), 0);
    }
}
