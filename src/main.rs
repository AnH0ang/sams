use std::process::ExitCode;

use clap::Parser;
use colored::Colorize;
use sams::args::Args;
use sams::run;

fn main() -> ExitCode {
    let args = Args::parse();

    match run(args) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}{:?}", "error: ".red().bold(), err);
            ExitCode::FAILURE
        },
    }
}
