mod json_schema;
mod migration;

use colored::*;
use std::path::Path;

use core_access::event_schema::{/*PermissionSetEvent, RoleEvent,*/ UserEvent};
// use core_accounting::event_schema::{AccountingCsvEvent, ChartEvent, ManualTransactionEvent};
// use core_credit::event_schema::{
//     CollateralEvent, CreditFacilityEvent, DisbursalEvent, InterestAccrualCycleEvent,
//     ObligationEvent, PaymentAllocationEvent, PaymentEvent, TermsTemplateEvent,
// };
// use core_custody::event_schema::CustodianEvent;
// use core_customer::event_schema::CustomerEvent;
// use core_deposit::event_schema::{DepositAccountEvent, DepositEvent, WithdrawalEvent};
// use governance::event_schema::{ApprovalProcessEvent, CommitteeEvent, PolicyEvent};
use schemars::schema_for;

pub use json_schema::*;
pub use migration::*;

pub struct SchemaInfo {
    pub name: &'static str,
    pub filename: &'static str,
    pub generate_schema: fn() -> serde_json::Value,
}

pub fn update_schemas(schemas_out_dir: &str, migrations_out_dir: &str) -> anyhow::Result<()> {
    let schemas = vec![
        SchemaInfo {
            name: "UserEvent",
            filename: "user_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(UserEvent)).unwrap(),
        },
        // SchemaInfo {
        //     name: "RoleEvent",
        //     filename: "role_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(RoleEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "PermissionSetEvent",
        //     filename: "permission_set_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(PermissionSetEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "ApprovalProcessEvent",
        //     filename: "approval_process_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(ApprovalProcessEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "CommitteeEvent",
        //     filename: "committee_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(CommitteeEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "PolicyEvent",
        //     filename: "policy_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(PolicyEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "CustodianEvent",
        //     filename: "custodian_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(CustodianEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "CustomerEvent",
        //     filename: "customer_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(CustomerEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "DepositAccountEvent",
        //     filename: "deposit_account_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(DepositAccountEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "DepositEvent",
        //     filename: "deposit_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(DepositEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "WithdrawalEvent",
        //     filename: "withdrawal_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(WithdrawalEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "CollateralEvent",
        //     filename: "collateral_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(CollateralEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "CreditFacilityEvent",
        //     filename: "credit_facility_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(CreditFacilityEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "DisbursalEvent",
        //     filename: "disbursal_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(DisbursalEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "InterestAccrualCycleEvent",
        //     filename: "interest_accrual_cycle_event_schema.json",
        //     generate_schema: || {
        //         serde_json::to_value(schema_for!(InterestAccrualCycleEvent)).unwrap()
        //     },
        // },
        // SchemaInfo {
        //     name: "ObligationEvent",
        //     filename: "obligation_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(ObligationEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "PaymentEvent",
        //     filename: "payment_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(PaymentEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "PaymentAllocationEvent",
        //     filename: "payment_allocation_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(PaymentAllocationEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "TermsTemplateEvent",
        //     filename: "terms_template_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(TermsTemplateEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "ChartEvent",
        //     filename: "chart_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(ChartEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "AccountingCsvEvent",
        //     filename: "accounting_csv_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(AccountingCsvEvent)).unwrap(),
        // },
        // SchemaInfo {
        //     name: "ManualTransactionEvent",
        //     filename: "manual_transaction_event_schema.json",
        //     generate_schema: || serde_json::to_value(schema_for!(ManualTransactionEvent)).unwrap(),
        // },
    ];

    process_schemas(&schemas, schemas_out_dir)?;

    // Generate migrations for rollup tables
    println!(
        "\n{} Generating rollup table migrations...",
        "🔨".blue().bold()
    );
    let schemas_dir = Path::new(schemas_out_dir);
    generate_rollup_migrations(&schemas, &schemas_dir, migrations_out_dir)?;

    Ok(())
}
