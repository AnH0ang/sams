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
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Sync dotfiles
    Sync(SyncArgs),

    /// Initialize a new dotfile configuration
    Init(InitArgs),

    /// Clone a dotfile configuration
    Clone(CloneArgs),

    /// (Plumbing) Interactively ask for dotfile configurations
    Ask(AskArgs),

    /// (Plumbing) Render dotfile templates
    Render,

    /// (Plumbing) Link files
    Link,

    /// (Plumbing) Run install scripts
    Install,

    /// (Plumbing) Pull
    Pull,

    /// (Plumbing) Generate shell completions
    Completions {
        /// The shell to generate the completions for
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },

    /// Generate json schema for `sams.toml` config file
    #[command(hide = true)]
    JsonSchema(JsonSchemaArgs),
}

#[derive(Clone, Debug, clap::Parser)]
pub struct GlobalArgs {
    /// Config file
    #[clap(short, long = "config", default_value = "sams.toml")]
    pub config_path: PathBuf,

    /// Root directory
    #[clap(short, long, default_value = ".")]
    pub root: PathBuf,
}

#[derive(Clone, Debug, clap::Parser)]
pub struct SyncArgs {
    // Overwrite existing answers file
    #[clap(long)]
    pub ask: bool,
}

#[derive(Clone, Debug, clap::Parser)]
pub struct CloneArgs {
    // Git repository URL
    pub url: String,

    // destination directory
    #[clap(short, long, default_value = "~/.config")]
    pub dest: PathBuf,
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
pub struct JsonSchemaArgs {
    /// Output file
    #[arg(short, long, default_value = "sams.schema.json")]
    pub file: PathBuf,
}
