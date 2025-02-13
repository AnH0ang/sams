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

pub fn run(
    Args {
        global_args,
        command,
    }: Args,
) -> Result<ExitCode> {
    match command {
        Commands::Sync(args) => {
            commands::sync::sync(args, &global_args)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Init(args) => {
            commands::init::init(args)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Ask(arg) => {
            commands::ask::ask(arg, &global_args)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Render => {
            commands::render::render(&global_args)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Link => {
            commands::link::link(&global_args)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Completions { shell } => {
            shell.generate(&mut Args::command(), &mut std::io::stdout());
            Ok(ExitCode::SUCCESS)
        },
        Commands::Install => {
            commands::install::install(&global_args)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::GenerateJsonSchema(args) => {
            commands::generate_json_schema::generate_json_schema(args)?;
            Ok(ExitCode::SUCCESS)
        },
    }
}
