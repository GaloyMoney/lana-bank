use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::{cli::WorkflowAction, output};

const SCHEMA_GRAPHQL: &str = include_str!("../../../lana/admin-server/src/graphql/schema.graphql");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SchemaContainer {
    Query,
    Mutation,
}

#[derive(Debug, Clone)]
struct WorkflowDefinition {
    steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone)]
struct WorkflowStep {
    id: String,
    operation: String,
    command: String,
    description: String,
    requires: Vec<String>,
    produces: Vec<AutomationToken>,
    depends_on: Vec<String>,
    mutating: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AutomationDefinition {
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    produces: Vec<AutomationToken>,
}

#[derive(Debug, Clone)]
struct ParsedWorkflowDoc {
    description: String,
    automation: AutomationDefinition,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AutomationToken {
    token: String,
    path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowDeps<'a> {
    target_step: &'a str,
    include_read_only: bool,
    steps: Vec<WorkflowStepView<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowStepView<'a> {
    index: usize,
    id: &'a str,
    operation: &'a str,
    command: &'a str,
    description: &'a str,
    requires: &'a [String],
    produces: Vec<&'a str>,
    depends_on: Vec<&'a str>,
    mutating: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowExport<'a> {
    source: &'a str,
    steps: Vec<WorkflowExportStep<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowExportStep<'a> {
    id: &'a str,
    operation: &'a str,
    command: &'a str,
    description: &'a str,
    kind: &'static str,
    requires: &'a [String],
    produces: Vec<&'a str>,
    depends_on: &'a [String],
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkflowVerifyReport {
    ok: bool,
    step_count: usize,
    token_count: usize,
    errors: Vec<String>,
}

pub fn execute(action: WorkflowAction, json: bool) -> anyhow::Result<()> {
    match action {
        WorkflowAction::List { yaml } => workflow_list(json, yaml),
        WorkflowAction::Deps { step, all } => workflow_deps(&step, all, json),
        WorkflowAction::Verify => workflow_verify(json),
    }
}

fn workflow_list(json: bool, yaml: bool) -> anyhow::Result<()> {
    if json && yaml {
        bail!("`workflow list` accepts either --json or --yaml, not both");
    }

    let workflow = load_workflow()?;
    let full_graph = build_workflow_export(&workflow)?;

    if json {
        return output::print_json(&full_graph);
    }
    if yaml {
        print!("{}", serde_yaml::to_string(&full_graph)?);
        return Ok(());
    }

    let rows = full_graph
        .steps
        .iter()
        .map(|step| {
            vec![
                step.id.to_string(),
                step.operation.to_string(),
                step.command.to_string(),
                step.description.to_string(),
                step.kind.to_string(),
                if step.depends_on.is_empty() {
                    "-".to_string()
                } else {
                    step.depends_on.join(", ")
                },
            ]
        })
        .collect();
    output::print_table(
        &[
            "Step ID",
            "Operation",
            "Command",
            "Description",
            "Type",
            "Depends On",
        ],
        rows,
    );
    Ok(())
}

fn workflow_deps(target_step: &str, include_read_only: bool, json: bool) -> anyhow::Result<()> {
    let workflow = load_workflow()?;
    let steps = collect_step_views(&workflow, Some(target_step), include_read_only)?;
    let deps = WorkflowDeps {
        target_step,
        include_read_only,
        steps,
    };

    if json {
        return output::print_json(&deps);
    }

    println!("Target Step: {}", deps.target_step);
    println!(
        "Include Read Only: {}",
        if deps.include_read_only {
            "true"
        } else {
            "false"
        }
    );
    println!("Required Steps:");

    if deps.steps.is_empty() {
        println!("  (no matching steps after filtering)");
        return Ok(());
    }

    for step in deps.steps {
        println!(
            "  {:>2}. {} [{}] -> {}",
            step.index, step.id, step.operation, step.command
        );
        if !step.description.is_empty() {
            println!("      Description: {}", step.description);
        }
        println!(
            "      Requires: {}",
            if step.requires.is_empty() {
                "-".to_string()
            } else {
                step.requires.join(", ")
            }
        );
        println!(
            "      Produces: {}",
            if step.produces.is_empty() {
                "-".to_string()
            } else {
                step.produces.join(", ")
            }
        );
    }

    Ok(())
}

fn build_workflow_export(workflow: &WorkflowDefinition) -> anyhow::Result<WorkflowExport<'_>> {
    let step_by_id: BTreeMap<&str, &WorkflowStep> = workflow
        .steps
        .iter()
        .map(|step| (step.id.as_str(), step))
        .collect();
    let included_ids: BTreeSet<&str> = workflow.steps.iter().map(|step| step.id.as_str()).collect();
    let ordered_ids = topologically_order_steps(workflow, &step_by_id, &included_ids)?;
    Ok(WorkflowExport {
        source: "schema.graphql workflow metadata",
        steps: ordered_ids
            .into_iter()
            .map(|step_id| {
                let step = step_by_id
                    .get(step_id)
                    .copied()
                    .expect("topologically ordered workflow step should exist");
                WorkflowExportStep {
                    id: &step.id,
                    operation: &step.operation,
                    command: &step.command,
                    description: &step.description,
                    kind: step_kind(step.mutating),
                    requires: &step.requires,
                    produces: step
                        .produces
                        .iter()
                        .map(|token| token.token.as_str())
                        .collect(),
                    depends_on: &step.depends_on,
                }
            })
            .collect(),
    })
}

fn step_kind(mutating: bool) -> &'static str {
    if mutating { "mutation" } else { "query" }
}

fn verify_workflow_contract(workflow: &WorkflowDefinition) -> Vec<String> {
    let mut errors = Vec::new();
    let mut step_ids = BTreeSet::new();
    let mut operations = BTreeSet::new();
    let mut token_producers: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let required_tokens: BTreeSet<&str> = workflow
        .steps
        .iter()
        .flat_map(|step| step.requires.iter().map(String::as_str))
        .collect();

    for step in &workflow.steps {
        if !step_ids.insert(step.id.as_str()) {
            errors.push(format!("duplicate workflow step id `{}`", step.id));
        }
        if !operations.insert(step.operation.as_str()) {
            errors.push(format!(
                "duplicate workflow operation `{}` in schema metadata",
                step.operation
            ));
        }
        for dependency in &step.depends_on {
            if workflow
                .steps
                .iter()
                .all(|candidate| candidate.id != *dependency)
            {
                errors.push(format!(
                    "step `{}` depends on missing step `{dependency}`",
                    step.id
                ));
            }
        }
        for token in &step.produces {
            token_producers
                .entry(token.token.as_str())
                .or_default()
                .push(step.id.as_str());
        }
    }

    for (token, producers) in token_producers {
        if required_tokens.contains(token) && producers.len() > 1 {
            errors.push(format!(
                "required token `{token}` is produced by multiple steps: {}",
                producers.join(", ")
            ));
        }
    }

    let step_by_id: BTreeMap<&str, &WorkflowStep> = workflow
        .steps
        .iter()
        .map(|step| (step.id.as_str(), step))
        .collect();
    let included_ids: BTreeSet<&str> = workflow.steps.iter().map(|step| step.id.as_str()).collect();
    if let Err(err) = topologically_order_steps(workflow, &step_by_id, &included_ids) {
        errors.push(format!("workflow graph is not a valid DAG: {err}"));
    }

    for step in &workflow.steps {
        for required in &step.requires {
            if !step.depends_on.iter().any(|dependency| {
                step_by_id[dependency.as_str()]
                    .produces
                    .iter()
                    .any(|produced| produced.token == *required || produced.path == *required)
            }) {
                errors.push(format!(
                    "step `{}` requires token `{required}` but no dependency produces it",
                    step.id
                ));
            }
        }
        for token in &step.produces {
            if token.path.trim().is_empty() {
                errors.push(format!(
                    "step `{}` declares token `{}` with an empty schema path",
                    step.id, token.token
                ));
            }
            if token.path.starts_with('.')
                || token.path.split('.').any(|part| part.trim().is_empty())
            {
                errors.push(format!(
                    "step `{}` declares token `{}` with invalid path `{}`",
                    step.id, token.token, token.path
                ));
            }
        }
        if step.description.trim().is_empty() {
            errors.push(format!(
                "step `{}` is missing a human description in schema docs",
                step.id
            ));
        }
        if operation_to_command(&step.operation).is_err() {
            errors.push(format!(
                "step `{}` has no lana-admin command mapping for `{}`",
                step.id, step.operation
            ));
        }
    }

    errors.sort();
    errors.dedup();
    errors
}

fn workflow_verify(json: bool) -> anyhow::Result<()> {
    let workflow = load_workflow()?;
    let errors = verify_workflow_contract(&workflow);
    let report = WorkflowVerifyReport {
        ok: errors.is_empty(),
        step_count: workflow.steps.len(),
        token_count: workflow.steps.iter().map(|step| step.produces.len()).sum(),
        errors,
    };

    if json {
        output::print_json(&report)?;
    } else if report.ok {
        println!(
            "Workflow metadata verified: {} steps, {} produced tokens",
            report.step_count, report.token_count
        );
    } else {
        println!(
            "Workflow metadata verification failed: {} error(s)",
            report.errors.len()
        );
        for error in &report.errors {
            println!("  - {error}");
        }
    }

    if report.ok {
        Ok(())
    } else {
        bail!(
            "workflow verification failed with {} error(s)",
            report.errors.len()
        )
    }
}

fn load_workflow() -> anyhow::Result<WorkflowDefinition> {
    parse_schema_workflow(SCHEMA_GRAPHQL)
}

fn parse_schema_workflow(schema: &str) -> anyhow::Result<WorkflowDefinition> {
    let mut steps = Vec::new();
    let mut container = None;
    let mut pending_description = None;
    let mut lines = schema.lines();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed == "type Query {" {
            container = Some(SchemaContainer::Query);
            pending_description = None;
            continue;
        }
        if trimmed == "type Mutation {" {
            container = Some(SchemaContainer::Mutation);
            pending_description = None;
            continue;
        }
        if container.is_some() && trimmed == "}" {
            container = None;
            pending_description = None;
            continue;
        }

        let Some(current_container) = container else {
            continue;
        };

        if trimmed.starts_with("\"\"\"") {
            pending_description = Some(collect_description(trimmed, &mut lines)?);
            continue;
        }

        let Some(operation) = parse_field_name(trimmed) else {
            continue;
        };

        let Some(description) = pending_description.take() else {
            continue;
        };
        let Some(parsed_doc) = parse_automation_definition(&description)? else {
            continue;
        };

        steps.push((current_container, operation.to_string(), parsed_doc));
    }

    if steps.is_empty() {
        bail!("no workflow metadata blocks found in embedded schema.graphql");
    }

    build_workflow_definition(steps)
}

fn collect_description<'a>(
    first_trimmed_line: &'a str,
    lines: &mut std::str::Lines<'a>,
) -> anyhow::Result<String> {
    let mut body = String::new();
    let first = first_trimmed_line
        .strip_prefix("\"\"\"")
        .expect("description should start with triple quote");

    if let Some((content, _)) = first.split_once("\"\"\"") {
        return Ok(content.trim_end().to_string());
    }

    if !first.is_empty() {
        body.push_str(first);
        body.push('\n');
    }

    for line in lines {
        if let Some((content, _)) = line.split_once("\"\"\"") {
            body.push_str(content);
            return Ok(body.trim_end().to_string());
        }

        body.push_str(line);
        body.push('\n');
    }

    bail!("unterminated triple-quoted description in schema.graphql")
}

fn parse_field_name(line: &str) -> Option<&str> {
    if line.is_empty() || line.starts_with('"') {
        return None;
    }

    let name = line.split(['(', ':']).next()?.trim();
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return None;
    }

    Some(name)
}

fn parse_automation_definition(description: &str) -> anyhow::Result<Option<ParsedWorkflowDoc>> {
    let description = dedent_block(description);
    let lines: Vec<&str> = description.lines().collect();
    let Some(metadata_index) = lines.iter().position(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("requires:") || trimmed == "produces:"
    }) else {
        return Ok(None);
    };

    let block = lines[metadata_index..].join("\n");
    let description = lines[..metadata_index].join("\n").trim().to_string();
    let mut requires = Vec::new();
    let mut produces = Vec::new();
    enum Section {
        None,
        Requires,
        Produces,
    }
    let mut section = Section::None;

    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("```") {
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("requires:") {
            let value = value.trim();
            if value.is_empty() {
                section = Section::Requires;
            } else {
                requires = parse_token_list(value).with_context(|| {
                    format!("failed to parse requires in automation block:\n{block}")
                })?;
                section = Section::None;
            }
            continue;
        }

        if trimmed == "produces:" {
            section = Section::Produces;
            continue;
        }

        let Some(entry) = trimmed.strip_prefix("- ") else {
            bail!("unsupported automation line `{trimmed}` in block:\n{block}");
        };

        match section {
            Section::Requires => {
                requires.push(entry.trim().to_string());
            }
            Section::Produces => {
                produces.push(parse_produced_token(entry).with_context(|| {
                    format!(
                        "failed to parse produces entry `{entry}` in automation block:\n{block}"
                    )
                })?);
            }
            Section::None => {
                bail!("list item outside requires/produces in automation block:\n{block}");
            }
        }
    }

    Ok(Some(ParsedWorkflowDoc {
        description,
        automation: AutomationDefinition { requires, produces },
    }))
}

fn dedent_block(block: &str) -> String {
    let lines: Vec<&str> = block
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .collect();

    let indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|ch| ch.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    lines
        .into_iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                line.chars().skip(indent).collect::<String>()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_token_list(value: &str) -> anyhow::Result<Vec<String>> {
    let value = value.trim();
    if value == "[]" {
        return Ok(Vec::new());
    }

    let inner = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .with_context(|| format!("expected bracketed token list, got `{value}`"))?;

    Ok(inner
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn parse_produced_token(entry: &str) -> anyhow::Result<AutomationToken> {
    let (token, path) = if let Some((token, path)) = entry.split_once("<-") {
        (token.trim().to_string(), path.trim().to_string())
    } else {
        let path = entry.trim().to_string();
        let token = path
            .rsplit('.')
            .next()
            .filter(|token| !token.trim().is_empty())
            .map(str::to_string)
            .with_context(|| format!("expected `<path>` or `<token> <- <path>`, got `{entry}`"))?;
        (token, path)
    };

    if token.is_empty() || path.is_empty() {
        bail!("expected non-empty token and path in `{entry}`");
    }

    Ok(AutomationToken { token, path })
}

fn build_workflow_definition(
    raw_steps: Vec<(SchemaContainer, String, ParsedWorkflowDoc)>,
) -> anyhow::Result<WorkflowDefinition> {
    let mut token_producers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut path_producers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (_, operation, parsed_doc) in &raw_steps {
        let step_id = derive_step_id(operation);
        for token in &parsed_doc.automation.produces {
            token_producers
                .entry(token.token.clone())
                .or_default()
                .push(step_id.clone());
            path_producers
                .entry(token.path.clone())
                .or_default()
                .push(step_id.clone());
        }
    }

    let mut steps = Vec::with_capacity(raw_steps.len());
    for (container, operation, parsed_doc) in raw_steps {
        let step_id = derive_step_id(&operation);
        let mut depends_on = BTreeSet::new();
        for required in &parsed_doc.automation.requires {
            let producers = if required.contains('.') {
                &path_producers
            } else {
                &token_producers
            };
            let Some(step_ids) = producers.get(required) else {
                bail!(
                    "Automation step `{step_id}` requires token `{required}` with no producer in schema docs",
                );
            };
            if step_ids.len() != 1 {
                bail!(
                    "Automation step `{step_id}` requires token `{required}` with multiple producers: {}",
                    step_ids.join(", ")
                );
            }
            depends_on.insert(step_ids[0].clone());
        }

        for token in &parsed_doc.automation.produces {
            if token.path.trim().is_empty() {
                bail!(
                    "Automation step `{step_id}` has an empty path for produced token `{}`",
                    token.token
                );
            }
        }

        steps.push(WorkflowStep {
            id: step_id,
            operation: operation.clone(),
            command: operation_to_command(&operation)
                .with_context(|| format!("missing lana-admin command mapping for `{operation}`"))?,
            description: parsed_doc.description,
            requires: parsed_doc.automation.requires,
            produces: parsed_doc.automation.produces,
            depends_on: depends_on.into_iter().collect(),
            mutating: container == SchemaContainer::Mutation,
        });
    }

    Ok(WorkflowDefinition { steps })
}

fn derive_step_id(operation: &str) -> String {
    let mut out = String::with_capacity(operation.len() + 8);
    for (i, ch) in operation.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn operation_to_command(operation: &str) -> anyhow::Result<String> {
    let command = match operation {
        "prospectCreate" => "prospect create",
        "prospectConvert" => "prospect convert",
        "depositRecord" => "deposit record",
        "depositAccountCreate" => "deposit account create",
        "withdrawalInitiate" => "deposit withdrawal initiate",
        "withdrawalConfirm" => "deposit withdrawal confirm",
        "termsTemplateCreate" => "credit terms-template create",
        "creditFacilityProposalCreate" => "credit facility proposal-create",
        "creditFacilityProposalCustomerApprovalConclude" => "credit facility proposal-conclude",
        "pendingCreditFacility" => "credit facility pending-get",
        "collateralUpdate" => "credit collateral update",
        "creditFacilityDisbursalInitiate" => "credit facility disbursal-initiate",
        "creditFacilityAgreementGenerate" => "credit loan-agreement generate",
        "loanAgreementDownloadLinkGenerate" => "credit loan-agreement download-link",
        _ => bail!("unknown workflow command mapping"),
    };

    Ok(command.to_string())
}

fn collect_step_views<'a>(
    workflow: &'a WorkflowDefinition,
    target_step: Option<&str>,
    include_read_only: bool,
) -> anyhow::Result<Vec<WorkflowStepView<'a>>> {
    let step_by_id: BTreeMap<&str, &WorkflowStep> = workflow
        .steps
        .iter()
        .map(|step| (step.id.as_str(), step))
        .collect();

    let needed: BTreeSet<&str> = if let Some(target_step) = target_step {
        if !step_by_id.contains_key(target_step) {
            bail!(
                "step `{target_step}` not found in schema-derived dependency graph. Available steps: {}",
                workflow
                    .steps
                    .iter()
                    .map(|step| step.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        let mut needed = BTreeSet::new();
        let mut visiting = BTreeSet::new();
        collect_needed_steps(target_step, &step_by_id, &mut needed, &mut visiting)?;
        needed
    } else {
        workflow.steps.iter().map(|step| step.id.as_str()).collect()
    };

    let included_ids: BTreeSet<&str> = workflow
        .steps
        .iter()
        .filter(|step| {
            needed.contains(step.id.as_str())
                && (include_read_only || step.mutating || target_step == Some(step.id.as_str()))
        })
        .map(|step| step.id.as_str())
        .collect();

    let ordered_ids = topologically_order_steps(workflow, &step_by_id, &included_ids)?;
    let mut output_steps = Vec::new();
    for (index, step_id) in ordered_ids.into_iter().enumerate() {
        let step = step_by_id
            .get(step_id)
            .copied()
            .with_context(|| format!("unknown ordered workflow step `{step_id}`"))?;
        output_steps.push(WorkflowStepView {
            index: index + 1,
            id: &step.id,
            operation: &step.operation,
            command: &step.command,
            description: &step.description,
            requires: &step.requires,
            produces: step
                .produces
                .iter()
                .map(|token| token.token.as_str())
                .collect(),
            depends_on: visible_dependencies_for_step(step, &step_by_id, &included_ids, workflow)?,
            mutating: step.mutating,
        });
    }

    Ok(output_steps)
}

fn topologically_order_steps<'a>(
    workflow: &'a WorkflowDefinition,
    step_by_id: &BTreeMap<&'a str, &'a WorkflowStep>,
    included_ids: &BTreeSet<&'a str>,
) -> anyhow::Result<Vec<&'a str>> {
    let mut ordered = Vec::new();
    let mut visited = BTreeSet::new();
    let mut visiting = BTreeSet::new();

    for step in &workflow.steps {
        let step_id = step.id.as_str();
        if included_ids.contains(step_id) {
            visit_step_for_order(
                step_id,
                step_by_id,
                included_ids,
                &mut visited,
                &mut visiting,
                &mut ordered,
            )?;
        }
    }

    Ok(ordered)
}

fn visit_step_for_order<'a>(
    step_id: &'a str,
    step_by_id: &BTreeMap<&'a str, &'a WorkflowStep>,
    included_ids: &BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
    visiting: &mut BTreeSet<&'a str>,
    ordered: &mut Vec<&'a str>,
) -> anyhow::Result<()> {
    if visited.contains(step_id) {
        return Ok(());
    }
    if !visiting.insert(step_id) {
        bail!("cycle detected while ordering workflow at step `{step_id}`");
    }

    let step = step_by_id
        .get(step_id)
        .copied()
        .with_context(|| format!("unknown workflow step `{step_id}`"))?;

    for dependency in &step.depends_on {
        let dependency = dependency.as_str();
        if included_ids.contains(dependency) {
            visit_step_for_order(
                dependency,
                step_by_id,
                included_ids,
                visited,
                visiting,
                ordered,
            )?;
        }
    }

    visiting.remove(step_id);
    visited.insert(step_id);
    ordered.push(step_id);
    Ok(())
}

fn collect_needed_steps<'a>(
    current_id: &'a str,
    step_by_id: &BTreeMap<&'a str, &'a WorkflowStep>,
    needed: &mut BTreeSet<&'a str>,
    visiting: &mut BTreeSet<&'a str>,
) -> anyhow::Result<()> {
    if needed.contains(current_id) {
        return Ok(());
    }
    if !visiting.insert(current_id) {
        bail!("cycle detected while traversing workflow at step `{current_id}`");
    }

    let step = step_by_id
        .get(current_id)
        .copied()
        .with_context(|| format!("unknown dependency step `{current_id}`"))?;

    for dependency in &step.depends_on {
        collect_needed_steps(dependency, step_by_id, needed, visiting)?;
    }

    visiting.remove(current_id);
    needed.insert(current_id);
    Ok(())
}

fn visible_dependencies_for_step<'a>(
    step: &'a WorkflowStep,
    step_by_id: &BTreeMap<&'a str, &'a WorkflowStep>,
    included_ids: &BTreeSet<&'a str>,
    workflow: &'a WorkflowDefinition,
) -> anyhow::Result<Vec<&'a str>> {
    let mut visible = BTreeSet::new();
    for dependency in &step.depends_on {
        collect_visible_dependency(
            dependency.as_str(),
            step_by_id,
            included_ids,
            &mut BTreeSet::new(),
            &mut visible,
        )?;
    }

    Ok(workflow
        .steps
        .iter()
        .map(|step| step.id.as_str())
        .filter(|id| visible.contains(id))
        .collect())
}

fn collect_visible_dependency<'a>(
    dependency_id: &'a str,
    step_by_id: &BTreeMap<&'a str, &'a WorkflowStep>,
    included_ids: &BTreeSet<&'a str>,
    visiting: &mut BTreeSet<&'a str>,
    visible: &mut BTreeSet<&'a str>,
) -> anyhow::Result<()> {
    if included_ids.contains(dependency_id) {
        visible.insert(dependency_id);
        return Ok(());
    }

    if !visiting.insert(dependency_id) {
        bail!("cycle detected while projecting filtered dependencies at step `{dependency_id}`");
    }

    let step = step_by_id
        .get(dependency_id)
        .copied()
        .with_context(|| format!("unknown projected dependency step `{dependency_id}`"))?;

    for dependency in &step.depends_on {
        collect_visible_dependency(
            dependency.as_str(),
            step_by_id,
            included_ids,
            visiting,
            visible,
        )?;
    }

    visiting.remove(dependency_id);
    Ok(())
}
