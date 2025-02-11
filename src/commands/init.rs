use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::args::InitArgs;
use crate::config::{Config, DataType, Parameter};

pub fn init(args: InitArgs) -> Result<()> {
    let config = default_config();
    let config_toml = toml::to_string(&config).context("Failed to serialize config to TOML")?;

    if !args.no_git {
        init_git_repo().context("Git repository check failed")?;
    }

    write_config(&args.file, &config_toml, args.force)
        .with_context(|| format!("Failed to initialize config file: {}", args.file.display()))
}

/// Initializes a Git repository if one does not already exist.
fn init_git_repo() -> Result<()> {
    if Path::new(".git").exists() {
        return Ok(());
    }

    let status = Command::new("git")
        .arg("init")
        .status()
        .context("Failed to execute `git init`")?;

    if status.success() {
        println!("Initialized empty Git repository.");
        Ok(())
    } else {
        bail!("Failed to initialize Git repository");
    }
}

/// Returns a default configuration.
fn default_config() -> Config {
    Config {
        parameters: vec![Parameter::Text {
            name: "user".into(),
            description: Some("Enter username".into()),
            default: None,
            placeholder: Some("username".into()),
            data_type: DataType::Str,
        }],
        ..Config::default()
    }
}

/// Writes configuration to a file, respecting the `force` flag.
fn write_config(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        bail!("File already exists: {}", path.display());
    }

    fs::write(path, content)
        .with_context(|| format!("Failed to write config to: {}", path.display()))
}
