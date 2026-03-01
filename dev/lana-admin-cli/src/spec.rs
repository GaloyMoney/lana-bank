use std::time::{SystemTime, UNIX_EPOCH};

use clap::CommandFactory;
use serde::Serialize;

use crate::cli::Cli;

#[derive(Serialize)]
struct CliSpec {
    generated_at_epoch_secs: u64,
    root: CommandSpec,
}

#[derive(Serialize)]
struct CommandSpec {
    path: String,
    name: String,
    about: Option<String>,
    long_about: Option<String>,
    command_kind: String,
    lifecycle_phase: String,
    mutating: bool,
    supported_environments: Vec<String>,
    permission_hint: Option<String>,
    output_id_fields: Vec<String>,
    preconditions: Vec<String>,
    args: Vec<ArgSpec>,
    subcommands: Vec<CommandSpec>,
}

#[derive(Serialize)]
struct ArgSpec {
    id: String,
    long_flag: Option<String>,
    short_flag: Option<String>,
    required: bool,
    takes_value: bool,
    global: bool,
    format_hint: Option<String>,
    default_values: Vec<String>,
    value_names: Vec<String>,
    possible_values: Vec<String>,
    help: Option<String>,
    long_help: Option<String>,
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn styled_to_string(s: Option<&clap::builder::StyledStr>) -> Option<String> {
    s.map(|v| v.to_string())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn infer_arg_format(
    long_name: Option<&str>,
    takes_value: bool,
    possible_values: &[String],
) -> Option<String> {
    if !possible_values.is_empty() {
        return Some("enum".to_string());
    }
    if !takes_value {
        return Some("boolean_flag".to_string());
    }

    let Some(long) = long_name else {
        return None;
    };

    if long.ends_with("-id") || long == "id" {
        return Some("uuid".to_string());
    }

    match long {
        "from" | "until" | "to" | "date" => Some("date_yyyy_mm_dd".to_string()),
        "effective" => Some("datetime_rfc3339".to_string()),
        "email" => Some("email".to_string()),
        "file" => Some("file_path".to_string()),
        "input-json" | "config-json" | "entries-json" | "value-json" => {
            Some("json_string".to_string())
        }
        "amount" | "facility-amount" | "annual-rate" | "one-time-fee-rate" | "initial-cvl"
        | "margin-call-cvl" | "liquidation-cvl" => Some("decimal_string".to_string()),
        _ => None,
    }
}

fn build_arg_spec(arg: &clap::Arg) -> ArgSpec {
    let possible_values: Vec<String> = arg
        .get_possible_values()
        .into_iter()
        .map(|v| v.get_name().to_string())
        .collect();

    let value_names = arg
        .get_value_names()
        .map(|vals| vals.iter().map(|v| v.to_string()).collect())
        .unwrap_or_default();

    let default_values = arg
        .get_default_values()
        .iter()
        .map(|v| v.to_string_lossy().to_string())
        .collect();

    let long_name = arg.get_long();
    let takes_value = arg.get_action().takes_values();

    ArgSpec {
        id: arg.get_id().to_string(),
        long_flag: long_name.map(|v| format!("--{v}")),
        short_flag: arg.get_short().map(|v| format!("-{v}")),
        required: arg.is_required_set(),
        takes_value,
        global: arg.is_global_set(),
        format_hint: infer_arg_format(long_name, takes_value, &possible_values),
        default_values,
        value_names,
        possible_values,
        help: styled_to_string(arg.get_help()),
        long_help: styled_to_string(arg.get_long_help()),
    }
}

fn action_name(path: &str) -> Option<&str> {
    path.split_whitespace().last()
}

fn command_kind(path: &str, is_leaf: bool) -> String {
    if !is_leaf {
        return "group".to_string();
    }

    match path {
        "lana-admin-cli login" | "lana-admin-cli logout" => "auth".to_string(),
        "lana-admin-cli spec" => "meta".to_string(),
        _ if is_mutating(path) => "mutation".to_string(),
        _ => "query".to_string(),
    }
}

fn lifecycle_phase(path: &str, is_leaf: bool) -> String {
    if !is_leaf {
        return "group".to_string();
    }
    if !is_mutating(path) {
        return "read_only".to_string();
    }

    let action = action_name(path).unwrap_or_default();
    match action {
        "close" | "close-month" | "freeze" | "archive" | "delete" | "cancel-withdrawal"
        | "revert-withdrawal" => "destructive_end_state".to_string(),
        "record-deposit"
        | "initiate-withdrawal"
        | "confirm-withdrawal"
        | "proposal-conclude"
        | "disbursal-initiate"
        | "partial-payment-record"
        | "record-collateral-sent"
        | "record-payment-received" => "stateful_mutation".to_string(),
        _ => "seed_or_setup_mutation".to_string(),
    }
}

fn is_mutating(path: &str) -> bool {
    let action = action_name(path).unwrap_or_default();
    matches!(
        action,
        "create"
            | "convert"
            | "close"
            | "close-month"
            | "record-deposit"
            | "initiate-withdrawal"
            | "confirm-withdrawal"
            | "cancel-withdrawal"
            | "revert-withdrawal"
            | "freeze"
            | "unfreeze"
            | "proposal-create"
            | "proposal-conclude"
            | "disbursal-initiate"
            | "partial-payment-record"
            | "approve"
            | "deny"
            | "record-collateral-sent"
            | "record-payment-received"
            | "add-root-node"
            | "add-child-node"
            | "csv-import"
            | "manual-transaction"
            | "create-ledger-csv"
            | "attach"
            | "archive"
            | "delete"
            | "update"
            | "trigger"
    )
}

fn permission_hint(path: &str) -> Option<String> {
    match path {
        "lana-admin-cli user create" => Some(
            "Requires operator role with user-management permissions; may return NotAuthorized for regular operators.".to_string(),
        ),
        "lana-admin-cli approval-process approve" | "lana-admin-cli approval-process deny" => {
            Some("Requires approval-decision permission for the process.".to_string())
        }
        "lana-admin-cli audit list" | "lana-admin-cli audit customer" => {
            Some("Requires audit-read access.".to_string())
        }
        _ => None,
    }
}

fn output_id_fields(path: &str) -> Vec<String> {
    let fields: &[&str] = match path {
        "lana-admin-cli prospect create" => &["prospectId", "publicId"],
        "lana-admin-cli prospect convert" => &["customerId", "publicId"],
        "lana-admin-cli deposit-account create" => &["depositAccountId", "publicId"],
        "lana-admin-cli deposit-account initiate-withdrawal" => {
            &["withdrawalId", "approvalProcessId"]
        }
        "lana-admin-cli terms-template create" => &["termsId"],
        "lana-admin-cli credit-facility proposal-create" => &["creditFacilityProposalId"],
        "lana-admin-cli credit-facility pending-get" => {
            &["pendingCreditFacilityId", "collateralId"]
        }
        "lana-admin-cli credit-facility disbursal-initiate" => &["disbursalId", "publicId"],
        "lana-admin-cli credit-facility find" => &["creditFacilityId", "collateralId"],
        "lana-admin-cli document attach" => &["documentId"],
        "lana-admin-cli document delete" => &["deletedDocumentId"],
        "lana-admin-cli csv-export create-ledger-csv" => &["documentId", "ledgerAccountId"],
        "lana-admin-cli loan-agreement generate" => &["id"],
        "lana-admin-cli fiscal-year list" => &["id", "fiscalYearId"],
        "lana-admin-cli report trigger" => &["runId"],
        _ => &[],
    };
    fields.iter().map(|v| (*v).to_string()).collect()
}

fn preconditions(path: &str) -> Vec<String> {
    let notes: &[&str] = match path {
        "lana-admin-cli prospect convert" => {
            &["Prospect must exist and be convertible (not already converted/closed)."]
        }
        "lana-admin-cli deposit-account record-deposit" => {
            &["Deposit account must be open (not frozen/closed)."]
        }
        "lana-admin-cli deposit-account initiate-withdrawal" => &[
            "Deposit account must be open (not frozen/closed).",
            "Velocity/risk checks may reject withdrawals.",
        ],
        "lana-admin-cli deposit-account confirm-withdrawal"
        | "lana-admin-cli deposit-account cancel-withdrawal"
        | "lana-admin-cli deposit-account revert-withdrawal" => {
            &["Withdrawal must be in a state compatible with the requested transition."]
        }
        "lana-admin-cli credit-facility proposal-conclude" => {
            &["Use proposal ID from proposal-create/proposal-list/proposal-get."]
        }
        "lana-admin-cli credit-facility disbursal-initiate" => {
            &["Credit facility must be ACTIVE before disbursal."]
        }
        "lana-admin-cli liquidation record-collateral-sent"
        | "lana-admin-cli liquidation record-payment-received" => {
            &["Credit facility must have an active liquidation."]
        }
        "lana-admin-cli fiscal-year close" | "lana-admin-cli fiscal-year close-month" => &[
            "Use raw UUID for --fiscal-year-id (entity references like 'FiscalYear:<uuid>' are not accepted).",
        ],
        "lana-admin-cli document attach" => {
            &["Customer must exist and file path must be readable."]
        }
        "lana-admin-cli report download-link" => {
            &["Use reportId from report list/find and an extension present in report files."]
        }
        _ => &[],
    };
    notes.iter().map(|v| (*v).to_string()).collect()
}

fn supported_environments() -> Vec<String> {
    vec!["local".to_string(), "qa".to_string(), "staging".to_string()]
}

fn build_command_spec(cmd: &clap::Command, parent_path: Option<&str>) -> CommandSpec {
    let path = match parent_path {
        Some(parent) if !parent.is_empty() => format!("{parent} {}", cmd.get_name()),
        _ => cmd.get_name().to_string(),
    };

    let args: Vec<ArgSpec> = cmd
        .get_arguments()
        .filter(|a| !a.is_hide_set())
        .map(build_arg_spec)
        .collect();

    let subcommands: Vec<CommandSpec> = cmd
        .get_subcommands()
        .filter(|c| !c.is_hide_set() && c.get_name() != "help")
        .map(|c| build_command_spec(c, Some(&path)))
        .collect();

    let is_leaf = subcommands.is_empty();

    CommandSpec {
        path: path.clone(),
        name: cmd.get_name().to_string(),
        about: styled_to_string(cmd.get_about()),
        long_about: styled_to_string(cmd.get_long_about()),
        command_kind: command_kind(&path, is_leaf),
        lifecycle_phase: lifecycle_phase(&path, is_leaf),
        mutating: is_leaf && is_mutating(&path),
        supported_environments: supported_environments(),
        permission_hint: permission_hint(&path),
        output_id_fields: output_id_fields(&path),
        preconditions: preconditions(&path),
        args,
        subcommands,
    }
}

pub fn print() -> anyhow::Result<()> {
    let root_cmd = Cli::command();
    let spec = CliSpec {
        generated_at_epoch_secs: now_epoch(),
        root: build_command_spec(&root_cmd, None),
    };
    println!("{}", serde_json::to_string_pretty(&spec)?);
    Ok(())
}
