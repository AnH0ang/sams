use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    version,
    about,
    long_about = None,
    propagate_version = true
)]
pub struct Args {
    #[clap(flatten)]
    pub global_args: GlobalArgs,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize a new dotfile configuration
    Init(InitArgs),

    /// Interactively ask for dotfile configurations
    Ask(AskArgs),

    /// Render dotfile templates
    Render(RenderArgs),

    /// Generate shell completions
    Completions {
        /// The shell to generate the completions for
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
}

#[derive(Clone, Debug, clap::Parser)]
pub struct GlobalArgs {
    /// Config file
    #[clap(short, long = "config", default_value = "sams.toml")]
    pub config_path: PathBuf,
}

#[derive(Clone, Debug, clap::Parser)]
pub struct InitArgs {
    #[arg(long)]
    pub no_git: bool,

    #[arg(long, short, default_value = "sams.toml")]
    pub file: PathBuf,

    #[arg(long)]
    pub force: bool,
}

#[derive(Clone, Debug, clap::Parser)]
pub struct AskArgs {
    #[arg(long)]
    pub force: bool,
}

#[derive(Clone, Debug, clap::Parser)]
pub struct RenderArgs {
    pub path: Option<PathBuf>,
}
