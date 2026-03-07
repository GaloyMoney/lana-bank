use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::{cli::WorkflowAction, output};

const WORKFLOW_YAML: &str = include_str!("../workflows/workflow.yml");

#[derive(Debug, Clone, Deserialize)]
struct WorkflowDefinition {
    steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, Deserialize)]
struct WorkflowStep {
    id: String,
    command: String,
    #[serde(default)]
    depends_on: Vec<String>,
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
    command: &'a str,
    depends_on: Vec<&'a str>,
    mutating: bool,
}

pub fn execute(action: WorkflowAction, json: bool) -> anyhow::Result<()> {
    match action {
        WorkflowAction::List => workflow_list(json),
        WorkflowAction::Deps { step, all } => workflow_deps(&step, all, json),
    }
}

fn workflow_list(json: bool) -> anyhow::Result<()> {
    let workflow = load_workflow()?;
    let steps = collect_step_views(&workflow, None, true)?;

    if json {
        return output::print_json(&steps);
    }

    let rows = steps
        .iter()
        .map(|step| {
            vec![
                step.index.to_string(),
                step.id.to_string(),
                step.command.to_string(),
                if step.mutating {
                    "mutation".to_string()
                } else {
                    "read".to_string()
                },
                if step.depends_on.is_empty() {
                    "-".to_string()
                } else {
                    step.depends_on.join(", ")
                },
            ]
        })
        .collect();
    output::print_table(&["#", "Step ID", "Command", "Type", "Depends On"], rows);
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
        println!("  {:>2}. {} -> {}", step.index, step.id, step.command);
    }

    Ok(())
}

fn load_workflow() -> anyhow::Result<WorkflowDefinition> {
    serde_yaml::from_str(WORKFLOW_YAML)
        .context("failed to parse embedded workflow dependency graph")
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
                "step `{target_step}` not found in embedded dependency graph. Available steps: {}",
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
                && (include_read_only || is_mutating_command(&step.command))
        })
        .map(|step| step.id.as_str())
        .collect();

    let mut index = 0usize;
    let mut output_steps = Vec::new();
    for step in &workflow.steps {
        if !included_ids.contains(step.id.as_str()) {
            continue;
        }

        index += 1;
        output_steps.push(WorkflowStepView {
            index,
            id: &step.id,
            command: &step.command,
            depends_on: visible_dependencies_for_step(step, &step_by_id, &included_ids, workflow)?,
            mutating: is_mutating_command(&step.command),
        });
    }

    Ok(output_steps)
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
        let dependency = dependency.as_str();
        if !step_by_id.contains_key(dependency) {
            bail!(
                "unknown dependency `{dependency}` referenced by step `{}`",
                step.id
            );
        }
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

fn is_mutating_command(command: &str) -> bool {
    let leaf = command.split_whitespace().last().unwrap_or(command);
    !matches!(
        leaf,
        "list"
            | "get"
            | "find"
            | "get-by-email"
            | "proposal-get"
            | "proposal-list"
            | "pending-get"
            | "download-link"
            | "account-entry"
            | "chart-of-accounts"
            | "base-config"
            | "credit-config"
            | "deposit-config"
            | "account-sets"
            | "ledger-account"
            | "balance-sheet"
            | "trial-balance"
            | "profit-and-loss"
            | "version"
            | "info"
    )
}
