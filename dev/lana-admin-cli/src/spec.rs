use clap::CommandFactory;
use serde::Serialize;

use crate::cli::Cli;

#[derive(Serialize)]
struct CliSpec {
    root: CommandSpec,
}

#[derive(Serialize)]
struct CommandSpec {
    path: String,
    name: String,
    about: Option<String>,
    long_about: Option<String>,
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
    default_values: Vec<String>,
    value_names: Vec<String>,
    possible_values: Vec<String>,
    help: Option<String>,
    long_help: Option<String>,
}

fn styled_to_string(s: Option<&clap::builder::StyledStr>) -> Option<String> {
    s.map(|v| v.to_string())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn build_arg_spec(arg: &clap::Arg) -> ArgSpec {
    let possible_values = arg
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

    ArgSpec {
        id: arg.get_id().to_string(),
        long_flag: arg.get_long().map(|v| format!("--{v}")),
        short_flag: arg.get_short().map(|v| format!("-{v}")),
        required: arg.is_required_set(),
        takes_value: arg.get_action().takes_values(),
        global: arg.is_global_set(),
        default_values,
        value_names,
        possible_values,
        help: styled_to_string(arg.get_help()),
        long_help: styled_to_string(arg.get_long_help()),
    }
}

fn build_command_spec(cmd: &clap::Command, parent_path: Option<&str>) -> CommandSpec {
    let path = match parent_path {
        Some(parent) if !parent.is_empty() => format!("{parent} {}", cmd.get_name()),
        _ => cmd.get_name().to_string(),
    };

    let args = cmd
        .get_arguments()
        .filter(|a| !a.is_hide_set())
        .map(build_arg_spec)
        .collect();

    let subcommands = cmd
        .get_subcommands()
        .filter(|c| !c.is_hide_set() && c.get_name() != "help")
        .map(|c| build_command_spec(c, Some(&path)))
        .collect();

    CommandSpec {
        path,
        name: cmd.get_name().to_string(),
        about: styled_to_string(cmd.get_about()),
        long_about: styled_to_string(cmd.get_long_about()),
        args,
        subcommands,
    }
}

pub fn print() -> anyhow::Result<()> {
    let root_cmd = Cli::command();
    let spec = CliSpec {
        root: build_command_spec(&root_cmd, None),
    };
    println!("{}", serde_json::to_string_pretty(&spec)?);
    Ok(())
}
