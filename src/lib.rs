use std::process::ExitCode;

use anyhow::Result;
use clap::CommandFactory;

use crate::args::{Args, Commands};

pub mod args;
pub mod commands;
pub mod config;

pub fn run(
    Args {
        global_args,
        command,
    }: Args,
) -> Result<ExitCode> {
    match command {
        Commands::Ask(arg) => {
            commands::ask::ask(arg, global_args)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Render(args) => {
            commands::render::render(args, global_args)?;
            Ok(ExitCode::SUCCESS)
        },
        Commands::Completions { shell } => {
            shell.generate(&mut Args::command(), &mut std::io::stdout());
            Ok(ExitCode::SUCCESS)
        },
    }
}
