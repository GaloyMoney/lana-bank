mod json_schema;
mod migration;

use colored::*;

use core_access::event_schema::{PermissionSetEvent, RoleEvent, UserEvent};
use core_accounting::event_schema::{
    ChartEvent, ChartNodeEvent, FiscalYearEvent, ManualTransactionEvent,
};
use core_credit::event_schema::{
    CollateralEvent, CreditFacilityEvent, CreditFacilityProposalEvent, DisbursalEvent,
    InterestAccrualCycleEvent, LiquidationEvent, ObligationEvent, PaymentAllocationEvent,
    PaymentEvent, PendingCreditFacilityEvent,
};
use core_credit_terms::TermsTemplateEvent;
use core_custody::event_schema::{CustodianEvent, WalletEvent};
use core_customer::event_schema::CustomerEvent;
use core_deposit::event_schema::{DepositAccountEvent, DepositEvent, WithdrawalEvent};
use core_report::event_schema::{ReportEvent, ReportRunEvent};
use document_storage::event_schema::DocumentEvent;
use domain_config::event_schema::DomainConfigEvent;
use governance::event_schema::{ApprovalProcessEvent, CommitteeEvent, PolicyEvent};
use schemars::schema_for;

pub use json_schema::{SchemaChangeInfo, detect_schema_changes, process_schemas_with_changes};
pub use migration::*;

#[derive(Clone)]
pub struct CollectionRollup {
    pub column_name: &'static str,
    pub values: &'static str,
    pub add_events: Vec<String>,
    pub remove_events: Vec<String>,
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

impl Default for SchemaInfo {
    fn default() -> Self {
        Self {
            name: "",
            filename: "",
            table_prefix: "core",
            collections: vec![],
            delete_events: vec![],
            toggle_events: vec![],
            generate_schema: || serde_json::Value::Null,
        }
    }
}

fn delete_related_migration_files(
    migrations_out_dir: &str,
    schema: &SchemaInfo,
) -> anyhow::Result<()> {
    let migrations_dir = std::path::Path::new(migrations_out_dir);
    if !migrations_dir.exists() {
        return Ok(());
    }

    // Generate the rollup table name pattern from the schema name
    // e.g., UserEvent -> core_user_events_rollup
    let entity_base = schema.name.replace("Event", "");
    let table_base = format!("{}_{}", schema.table_prefix, to_snake_case(&entity_base));
    let rollup_table_name = format!("{table_base}_events_rollup");

    // Read directory and find matching migration files
    let entries = std::fs::read_dir(migrations_dir)?;
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Check if this migration file is related to our rollup table
        // Pattern: YYYYMMDDHHMMSS_create_<rollup_table_name>.sql
        // Pattern: YYYYMMDDHHMMSS_update_<rollup_table_name>.sql
        if file_name_str.contains(&rollup_table_name)
            && (file_name_str.contains("_create_") || file_name_str.contains("_update_"))
            && file_name_str.ends_with(".sql")
        {
            let file_path = entry.path();
            std::fs::remove_file(&file_path)?;
            println!("  Deleted migration: {}", file_path.display());
        }
    }

    Ok(())
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 && !prev_was_upper {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
        prev_was_upper = ch.is_uppercase();
    }

    result
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
            generate_schema: || serde_json::to_value(schema_for!(UserEvent)).unwrap(),
            delete_events: vec!["RoleRevoked"],
            ..Default::default()
        },
        SchemaInfo {
            name: "RoleEvent",
            filename: "role_event_schema.json",
            collections: vec![CollectionRollup {
                column_name: "permission_set_ids",
                values: "permission_set_id",
                add_events: vec!["PermissionSetAdded".to_string()],
                remove_events: vec!["PermissionSetRemoved".to_string()],
            }],
            generate_schema: || serde_json::to_value(schema_for!(RoleEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "PermissionSetEvent",
            filename: "permission_set_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(PermissionSetEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "ApprovalProcessEvent",
            filename: "approval_process_event_schema.json",
            collections: vec![
                CollectionRollup {
                    column_name: "approver_ids",
                    values: "approver_id",
                    add_events: vec!["Approved".to_string()],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "denier_ids",
                    values: "denier_id",
                    add_events: vec!["Denied".to_string()],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "deny_reasons",
                    values: "reason",
                    add_events: vec!["Denied".to_string()],
                    remove_events: vec![],
                },
            ],
            toggle_events: vec!["Concluded"],
            generate_schema: || serde_json::to_value(schema_for!(ApprovalProcessEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "CommitteeEvent",
            filename: "committee_event_schema.json",
            collections: vec![CollectionRollup {
                column_name: "member_ids",
                values: "member_id",
                add_events: vec!["MemberAdded".to_string()],
                remove_events: vec!["MemberRemoved".to_string()],
            }],
            generate_schema: || serde_json::to_value(schema_for!(CommitteeEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "PolicyEvent",
            filename: "policy_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(PolicyEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "CustomerEvent",
            filename: "customer_event_schema.json",
            toggle_events: vec!["KycApproved"],
            generate_schema: || serde_json::to_value(schema_for!(CustomerEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "DepositAccountEvent",
            filename: "deposit_account_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(DepositAccountEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "DepositEvent",
            filename: "deposit_event_schema.json",
            collections: vec![CollectionRollup {
                column_name: "ledger_tx_ids",
                values: "ledger_tx_id",
                add_events: vec!["Initialized".to_string(), "Reverted".to_string()],
                remove_events: vec![],
            }],
            generate_schema: || serde_json::to_value(schema_for!(DepositEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "WithdrawalEvent",
            filename: "withdrawal_event_schema.json",
            collections: vec![CollectionRollup {
                column_name: "ledger_tx_ids",
                values: "ledger_tx_id",
                add_events: vec![
                    "Initialized".to_string(),
                    "Cancelled".to_string(),
                    "Confirmed".to_string(),
                    "Reverted".to_string(),
                ],
                remove_events: vec![],
            }],
            toggle_events: vec!["ApprovalProcessConcluded", "Confirmed", "Cancelled"],
            generate_schema: || serde_json::to_value(schema_for!(WithdrawalEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "CustodianEvent",
            filename: "custodian_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(CustodianEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "CollateralEvent",
            filename: "collateral_event_schema.json",
            collections: vec![CollectionRollup {
                column_name: "ledger_tx_ids",
                values: "ledger_tx_id",
                add_events: vec!["Updated".to_string()],
                remove_events: vec![],
            }],
            generate_schema: || serde_json::to_value(schema_for!(CollateralEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "CreditFacilityEvent",
            filename: "credit_facility_event_schema.json",
            collections: vec![
                CollectionRollup {
                    column_name: "ledger_tx_ids",
                    values: "ledger_tx_id",
                    add_events: vec![
                        "Initialized".to_string(),
                        "Activated".to_string(),
                        "InterestAccrualCycleConcluded".to_string(),
                    ],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "interest_accrual_ids",
                    values: "interest_accrual_id",
                    add_events: vec!["InterestAccrualCycleStarted".to_string()],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "obligation_ids",
                    values: "obligation_id",
                    add_events: vec!["InterestAccrualCycleConcluded".to_string()],
                    remove_events: vec![],
                },
            ],
            toggle_events: vec!["Completed", "Matured"],
            generate_schema: || serde_json::to_value(schema_for!(CreditFacilityEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "CreditFacilityProposalEvent",
            filename: "credit_facility_proposal_event_schema.json",
            toggle_events: vec!["ApprovalProcessConcluded"],
            generate_schema: || {
                serde_json::to_value(schema_for!(CreditFacilityProposalEvent)).unwrap()
            },
            ..Default::default()
        },
        SchemaInfo {
            name: "PendingCreditFacilityEvent",
            filename: "pending_credit_facility_schema.json",
            collections: vec![CollectionRollup {
                column_name: "ledger_tx_ids",
                values: "ledger_tx_id",
                add_events: vec!["Initialized".to_string()],
                remove_events: vec![],
            }],
            toggle_events: vec![
                "CollateralizationStateChanged",
                "CollateralizationRatioChanged",
                "Completed",
            ],
            generate_schema: || {
                serde_json::to_value(schema_for!(PendingCreditFacilityEvent)).unwrap()
            },
            ..Default::default()
        },
        SchemaInfo {
            name: "DisbursalEvent",
            filename: "disbursal_event_schema.json",
            toggle_events: vec!["ApprovalProcessConcluded", "Settled", "Cancelled"],
            generate_schema: || serde_json::to_value(schema_for!(DisbursalEvent)).unwrap(),
            collections: vec![CollectionRollup {
                column_name: "ledger_tx_ids",
                values: "ledger_tx_id",
                add_events: vec![
                    "Initialized".to_string(),
                    "Settled".to_string(),
                    "Cancelled".to_string(),
                ],
                remove_events: vec![],
            }],
            ..Default::default()
        },
        SchemaInfo {
            name: "InterestAccrualCycleEvent",
            filename: "interest_accrual_cycle_event_schema.json",
            collections: vec![CollectionRollup {
                column_name: "ledger_tx_ids",
                values: "ledger_tx_id",
                add_events: vec![
                    "InterestAccrued".to_string(),
                    "InterestAccrualsPosted".to_string(),
                ],
                remove_events: vec![],
            }],
            toggle_events: vec!["InterestAccrualsPosted"],
            generate_schema: || {
                serde_json::to_value(schema_for!(InterestAccrualCycleEvent)).unwrap()
            },
            ..Default::default()
        },
        SchemaInfo {
            name: "ObligationEvent",
            filename: "obligation_event_schema.json",
            collections: vec![
                CollectionRollup {
                    column_name: "ledger_tx_ids",
                    values: "ledger_tx_id",
                    add_events: vec![
                        "Initialized".to_string(),
                        "DueRecorded".to_string(),
                        "OverdueRecorded".to_string(),
                        "DefaultedRecorded".to_string(),
                        "PaymentAllocated".to_string(),
                    ],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "payment_allocation_ids",
                    values: "payment_allocation_id",
                    add_events: vec!["PaymentAllocated".to_string()],
                    remove_events: vec![],
                },
            ],
            toggle_events: vec![
                "DueRecorded",
                "OverdueRecorded",
                "DefaultedRecorded",
                "Completed",
            ],
            generate_schema: || serde_json::to_value(schema_for!(ObligationEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "PaymentEvent",
            filename: "payment_event_schema.json",
            toggle_events: vec![],
            generate_schema: || serde_json::to_value(schema_for!(PaymentEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "PaymentAllocationEvent",
            filename: "payment_allocation_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(PaymentAllocationEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "TermsTemplateEvent",
            filename: "terms_template_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(TermsTemplateEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "ChartEvent",
            filename: "chart_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(ChartEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "ChartNodeEvent",
            filename: "chart_node_event_schema.json",
            collections: vec![
                CollectionRollup {
                    column_name: "ledger_account_set_ids",
                    values: "ledger_account_set_id",
                    add_events: vec!["Initialized".to_string()],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "manual_ledger_account_ids",
                    values: "ledger_account_id",
                    add_events: vec!["ManualTransactionAccountAssigned".to_string()],
                    remove_events: vec![],
                },
                CollectionRollup {
                    column_name: "child_node_ids",
                    values: "child_node_id",
                    add_events: vec!["ChildNodeAdded".to_string()],
                    remove_events: vec![],
                },
            ],
            generate_schema: || serde_json::to_value(schema_for!(ChartNodeEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "LiquidationEvent",
            filename: "liquidation_event_schema.json",
            toggle_events: vec!["Completed"],
            generate_schema: || serde_json::to_value(schema_for!(LiquidationEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "DocumentEvent",
            filename: "document_event_schema.json",
            toggle_events: vec!["FileUploaded", "Deleted", "Archived"],
            generate_schema: || serde_json::to_value(schema_for!(DocumentEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "ManualTransactionEvent",
            filename: "manual_transaction_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(ManualTransactionEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "DomainConfigEvent",
            filename: "domain_config_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(DomainConfigEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "WalletEvent",
            filename: "wallet_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(WalletEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "FiscalYearEvent",
            filename: "fiscal_year_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(FiscalYearEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "ReportEvent",
            filename: "report_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(ReportEvent)).unwrap(),
            ..Default::default()
        },
        SchemaInfo {
            name: "ReportRunEvent",
            filename: "report_run_event_schema.json",
            generate_schema: || serde_json::to_value(schema_for!(ReportRunEvent)).unwrap(),
            ..Default::default()
        },
    ];

    // First, detect which schemas have changed
    let mut schema_changes = detect_schema_changes(&schemas, schemas_out_dir)?;

    // Delete existing schema files and migrations only for changed schemas if force_recreate is requested
    if force_recreate {
        let changed_schemas: Vec<_> = schema_changes
            .iter_mut()
            .filter(|change| change.has_changed)
            .collect();

        if !changed_schemas.is_empty() {
            println!(
                "{} Force recreate enabled - deleting schema files and migrations for {} changed schema(s)...",
                "🗑️".yellow().bold(),
                changed_schemas.len()
            );

            for change in changed_schemas {
                let schema_path =
                    std::path::Path::new(schemas_out_dir).join(change.schema_info.filename);
                if schema_path.exists() {
                    std::fs::remove_file(&schema_path)?;
                    println!("  Deleted schema: {}", schema_path.display());
                }

                change.previous_schema = None;

                // Delete related migration files
                delete_related_migration_files(migrations_out_dir, &change.schema_info)?;
            }
        } else {
            println!(
                "{} Force recreate enabled but no schema changes detected - nothing to delete",
                "ℹ️".blue().bold()
            );
        }
    }

    // Process schemas (this will now write files that were deleted above)
    let schema_changes = process_schemas_with_changes(&schema_changes, schemas_out_dir)?;

    // Generate migrations for rollup tables
    println!(
        "\n{} Generating rollup table migrations...",
        "🔨".blue().bold()
    );
    generate_rollup_migrations(&schema_changes, migrations_out_dir)?;

    Ok(())
}
