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

    /// Run install scripts
    Install(InstallArgs),

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
    /// Directory to initialize
    #[arg(default_value = ".")]
    pub dir: PathBuf,

    /// Do not initialize a git repository
    #[arg(long)]
    pub no_git: bool,

    /// Config file
    #[arg(long, short, default_value = "sams.toml")]
    pub file: PathBuf,

    /// Overwrite existing config file
    #[arg(long)]
    pub force: bool,
}

#[derive(Clone, Debug, clap::Parser)]
pub struct AskArgs {
    /// Overwrite existing answers file
    #[arg(long)]
    pub force: bool,
}

#[derive(Clone, Debug, clap::Parser)]
pub struct RenderArgs {
    /// Path to the template files
    pub path: Option<PathBuf>,
}

#[derive(Clone, Debug, clap::Parser)]
pub struct InstallArgs {}
