use std::process::ExitCode;

use anyhow::Result;
use clap::CommandFactory;

use crate::args::{Args, Commands};

pub mod args;
pub mod commands;
pub mod config;
pub mod context;
pub mod template;
pub mod walk;

pub fn run(Args { global, command }: Args) -> Result<ExitCode> {
    match command {
        Commands::Sync(args) => {
            commands::sync::sync(args, &global)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Init(args) => {
            commands::init::init(args)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Clone(args) => {
            commands::clone::clone(args, global)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Ask(arg) => {
            commands::ask::ask(arg, &global)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Render => {
            commands::render::render(&global)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Link => {
            commands::link::link(&global)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Completions { shell } => {
            shell.generate(&mut Args::command(), &mut std::io::stdout());
            Ok(ExitCode::SUCCESS)
        },
        Commands::Install => {
            commands::install::install(&global)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Pull => {
            commands::pull::pull(&global)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::JsonSchema(args) => {
            commands::jsonschema::generate_json_schema(args)?;
            Ok(ExitCode::SUCCESS)
        },
    }
}
