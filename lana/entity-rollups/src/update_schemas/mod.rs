mod json_schema;
mod migration;

use colored::*;

use core_access::event_schema::{PermissionSetEvent, RoleEvent, UserEvent};
use core_accounting::event_schema::{ChartEvent, ManualTransactionEvent};
use core_credit::event_schema::{
    CollateralEvent, CreditFacilityEvent, DisbursalEvent, InterestAccrualCycleEvent,
    ObligationEvent, PaymentAllocationEvent, PaymentEvent, TermsTemplateEvent,
};
use core_custody::event_schema::CustodianEvent;
use core_customer::event_schema::CustomerEvent;
use core_deposit::event_schema::{DepositAccountEvent, DepositEvent, WithdrawalEvent};
use governance::event_schema::{ApprovalProcessEvent, CommitteeEvent, PolicyEvent};
use schemars::schema_for;

pub use json_schema::*;
pub use migration::*;

#[derive(Clone)]
pub struct CollectionRollup {
    pub column_name: &'static str,
    pub values: &'static str,
    pub add_events: Vec<&'static str>,
    pub remove_events: Vec<&'static str>,
}

#[derive(Clone)]
pub struct SchemaInfo {
    pub name: &'static str,
    pub filename: &'static str,
    pub table_prefix: &'static str,
    pub collections: Vec<CollectionRollup>,
    pub delete_events: Vec<&'static str>,
    pub toggle_events: Vec<&'static str>,
    pub generate_schema: fn() -> serde_json::Value,
}

pub fn update_schemas(
    schemas_out_dir: &str,
    migrations_out_dir: &str,
    force_recreate: bool,
) -> anyhow::Result<()> {
    let schemas = vec![
        SchemaInfo {
            name: "UserEvent",
            filename: "user_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec!["RoleRevoked"],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(UserEvent)).unwrap(),
        },
        SchemaInfo {
            name: "RoleEvent",
            filename: "role_event_schema.json",
            table_prefix: "core",
            collections: vec![CollectionRollup {
                column_name: "permission_set_ids",
                values: "permission_set_id",
                add_events: vec!["PermissionSetAdded"],
                remove_events: vec!["PermissionSetRemoved"],
            }],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(RoleEvent)).unwrap(),
        },
        SchemaInfo {
            name: "PermissionSetEvent",
            filename: "permission_set_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(PermissionSetEvent)).unwrap(),
        },
        SchemaInfo {
            name: "ApprovalProcessEvent",
            table_prefix: "core",
            filename: "approval_process_event_schema.json",
            collections: vec![
                CollectionRollup {
                    column_name: "approver_ids",
                    values: "approver_id",
                    add_events: vec!["Approved"],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "denier_ids",
                    values: "denier_id",
                    add_events: vec!["Denied"],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "deny_reasons",
                    values: "reason",
                    add_events: vec!["Denied"],
                    remove_events: vec![],
                },
            ],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(ApprovalProcessEvent)).unwrap(),
        },
        SchemaInfo {
            name: "CommitteeEvent",
            table_prefix: "core",
            filename: "committee_event_schema.json",
            collections: vec![CollectionRollup {
                column_name: "member_ids",
                values: "member_id",
                add_events: vec!["MemberAdded"],
                remove_events: vec!["MemberRemoved"],
            }],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(CommitteeEvent)).unwrap(),
        },
        SchemaInfo {
            name: "PolicyEvent",
            filename: "policy_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(PolicyEvent)).unwrap(),
        },
        SchemaInfo {
            name: "CustomerEvent",
            filename: "customer_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec!["KycApproved"],
            generate_schema: || serde_json::to_value(schema_for!(CustomerEvent)).unwrap(),
        },
        SchemaInfo {
            name: "DepositAccountEvent",
            filename: "deposit_account_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(DepositAccountEvent)).unwrap(),
        },
        SchemaInfo {
            name: "DepositEvent",
            filename: "deposit_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(DepositEvent)).unwrap(),
        },
        SchemaInfo {
            name: "WithdrawalEvent",
            filename: "withdrawal_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec!["Confirmed", "Cancelled"],
            generate_schema: || serde_json::to_value(schema_for!(WithdrawalEvent)).unwrap(),
        },
        SchemaInfo {
            name: "CustodianEvent",
            filename: "custodian_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(CustodianEvent)).unwrap(),
        },
        SchemaInfo {
            name: "CollateralEvent",
            filename: "collateral_event_schema.json",
            table_prefix: "core",
            collections: vec![
                CollectionRollup {
                    column_name: "ledger_tx_ids",
                    values: "ledger_tx_id",
                    add_events: vec!["Updated"],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "diffs",
                    values: "abs_diff",
                    add_events: vec!["Updated"],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "actions",
                    values: "action",
                    add_events: vec!["Updated"],
                    remove_events: vec![],
                },
            ],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(CollateralEvent)).unwrap(),
        },
        SchemaInfo {
            name: "CreditFacilityEvent",
            filename: "credit_facility_event_schema.json",
            table_prefix: "core",
            collections: vec![
                CollectionRollup {
                    column_name: "interest_accrual_ids",
                    values: "interest_accrual_id",
                    add_events: vec!["InterestAccrualCycleStarted"],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "ledger_tx_ids",
                    values: "ledger_tx_id",
                    add_events: vec!["Initialized", "Activated", "InterestAccrualCycleConcluded"],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "obligation_ids",
                    values: "obligation_id",
                    add_events: vec!["InterestAccrualCycleConcluded"],
                    remove_events: vec![],
                },
            ],
            delete_events: vec![],
            toggle_events: vec!["ApprovalProcessConcluded", "Activated", "Completed"],
            generate_schema: || serde_json::to_value(schema_for!(CreditFacilityEvent)).unwrap(),
        },
        SchemaInfo {
            name: "DisbursalEvent",
            filename: "disbursal_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec!["ApprovalProcessConcluded", "Settled", "Cancelled"],
            generate_schema: || serde_json::to_value(schema_for!(DisbursalEvent)).unwrap(),
        },
        SchemaInfo {
            name: "InterestAccrualCycleEvent",
            filename: "interest_accrual_cycle_event_schema.json",
            table_prefix: "core",
            collections: vec![CollectionRollup {
                column_name: "ledger_tx_ids",
                values: "ledger_tx_id",
                add_events: vec!["InterestAccrued", "InterestAccrualsPosted"],
                remove_events: vec![],
            }],
            delete_events: vec![],
            toggle_events: vec!["InterestAccrualsPosted"],
            generate_schema: || {
                serde_json::to_value(schema_for!(InterestAccrualCycleEvent)).unwrap()
            },
        },
        SchemaInfo {
            name: "ObligationEvent",
            filename: "obligation_event_schema.json",
            table_prefix: "core",
            collections: vec![
                CollectionRollup {
                    column_name: "ledger_tx_ids",
                    values: "ledger_tx_id",
                    add_events: vec![
                        "Initialized",
                        "DueRecorded",
                        "OverdueRecorded",
                        "DefaultedRecorded",
                        "PaymentAllocated",
                    ],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "payment_ids",
                    values: "payment_id",
                    add_events: vec!["PaymentAllocated"],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "payment_allocation_ids",
                    values: "payment_allocation_id",
                    add_events: vec!["PaymentAllocated"],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "payment_allocation_amounts",
                    values: "payment_allocation_amount",
                    add_events: vec!["PaymentAllocated"],
                    remove_events: vec![],
                },
            ],
            delete_events: vec![],
            toggle_events: vec![
                "DueRecorded",
                "OverdueRecorded",
                "DefaultedRecorded",
                "Completed",
            ],
            generate_schema: || serde_json::to_value(schema_for!(ObligationEvent)).unwrap(),
        },
        SchemaInfo {
            name: "PaymentEvent",
            filename: "payment_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec!["PaymentAllocated"],
            generate_schema: || serde_json::to_value(schema_for!(PaymentEvent)).unwrap(),
        },
        SchemaInfo {
            name: "PaymentAllocationEvent",
            filename: "payment_allocation_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec!["PaymentAllocated"],
            generate_schema: || serde_json::to_value(schema_for!(PaymentAllocationEvent)).unwrap(),
        },
        SchemaInfo {
            name: "TermsTemplateEvent",
            filename: "terms_template_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(TermsTemplateEvent)).unwrap(),
        },
        SchemaInfo {
            name: "ChartEvent",
            filename: "chart_event_schema.json",
            table_prefix: "core",
            collections: vec![
                CollectionRollup {
                    column_name: "node_specs",
                    values: "spec",
                    add_events: vec!["NodeAdded"],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "ledger_account_set_ids",
                    values: "ledger_account_set_id",
                    add_events: vec!["NodeAdded"],
                    remove_events: vec![],
                },
            ],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(ChartEvent)).unwrap(),
        },
        SchemaInfo {
            name: "ManualTransactionEvent",
            filename: "manual_transaction_event_schema.json",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(ManualTransactionEvent)).unwrap(),
        },
    ];

    // Delete existing schema files if force_recreate is requested
    if force_recreate {
        println!(
            "{} Force recreate enabled - deleting existing schema files...",
            "🗑️".yellow().bold()
        );

        for schema in &schemas {
            let schema_path = std::path::Path::new(schemas_out_dir).join(schema.filename);
            if schema_path.exists() {
                std::fs::remove_file(&schema_path)?;
                println!("  Deleted: {}", schema_path.display());
            }
        }
    }

    let schema_changes = process_schemas(&schemas, schemas_out_dir)?;

    // Generate migrations for rollup tables
    println!(
        "\n{} Generating rollup table migrations...",
        "🔨".blue().bold()
    );
    generate_rollup_migrations(&schema_changes, migrations_out_dir)?;

    Ok(())
}
