use std::collections::HashMap;

use crate::{path::*, ChartId};

use super::ChartEvent;

pub struct ControlAccountAdded {
    name: String,
    path: ControlAccountPath,
}

#[derive(Debug)]
pub struct ControlAccountProjection {
    name: String,
    encoded_path: String,
    children: Vec<ControlSubAccountProjection>,
}

#[derive(Debug)]
pub struct ControlSubAccountProjection {
    name: String,
    encoded_path: String,
}

#[derive(Debug)]
pub struct ChartOfAccounts {
    id: ChartId,
    assets: Vec<ControlAccountProjection>,
    liabilities: Vec<ControlAccountProjection>,
    equity: Vec<ControlAccountProjection>,
    revenues: Vec<ControlAccountProjection>,
    expenses: Vec<ControlAccountProjection>,
}

pub(super) fn project<'a>(
    events: impl DoubleEndedIterator<Item = &'a ChartEvent>,
) -> ChartOfAccounts {
    let mut id: Option<ChartId> = None;
    let mut control_accounts_added: Vec<ControlAccountAdded> = vec![];
    let mut control_sub_accounts_by_parent: HashMap<String, Vec<ControlSubAccountProjection>> =
        HashMap::new();

    for event in events {
        match event {
            ChartEvent::Initialized { id: chart_id, .. } => id = Some(*chart_id),
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

    ChartOfAccounts {
        id: id.expect("Chart must be initialized"),
        assets: control_accounts_by_category
            .remove(&ChartCategory::Assets)
            .unwrap_or_default(),
        liabilities: control_accounts_by_category
            .remove(&ChartCategory::Liabilities)
            .unwrap_or_default(),
        equity: control_accounts_by_category
            .remove(&ChartCategory::Equity)
            .unwrap_or_default(),
        revenues: control_accounts_by_category
            .remove(&ChartCategory::Revenues)
            .unwrap_or_default(),
        expenses: control_accounts_by_category
            .remove(&ChartCategory::Expenses)
            .unwrap_or_default(),
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
            chart.chart().assets[0].children[0].encoded_path,
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
            chart.chart().liabilities[0].children[0].encoded_path,
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
        assert_eq!(chart.chart().equity[0].children[0].encoded_path, "30101");

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
        assert_eq!(chart.chart().revenues[0].encoded_path, "40100");
        assert_eq!(chart.chart().revenues[0].children.len(), 0);

        assert_eq!(chart.chart().expenses.len(), 0);
    }
}
