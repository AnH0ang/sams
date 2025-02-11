use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Ok, Result};

use crate::args::InitArgs;
use crate::config::{Config, DataType, Parameter};

pub fn init(args: InitArgs) -> Result<()> {
    let cfg = default_config();
    let cfg_dir = args.dir.join(args.file);
    let cfg_toml = toml::to_string(&cfg).context("Failed to serialize config to TOML")?;

    if !args.no_git {
        init_git_repo(&args.dir).context("Git repository check failed")?;
    }

    write_config(&cfg_dir, &cfg_toml, args.force)
        .with_context(|| format!("Failed to initialize config file: {}", cfg_dir.display()))
}

fn init_git_repo(directory: &Path) -> Result<()> {
    if directory.join(".git").exists() {
        return Ok(());
    }

    let status = Command::new("git")
        .current_dir(directory)
        .arg("init")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("Failed to execute `git init`")?;

    if status.success() {
        println!("Initialized empty Git repository.");
        Ok(())
    } else {
        bail!("Failed to initialize Git repository");
    }
}

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

fn write_config(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        bail!("File already exists: {}", path.display());
    }

    fs::write(path, content)
        .with_context(|| format!("Failed to write config to: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use tempfile::tempdir;

    use super::*;
    use crate::args::InitArgs;

    #[test]
    fn test_init_creates_config_file() {
        let temp_dir = tempdir().unwrap();
        env::set_current_dir(&temp_dir).unwrap();
        let config_file = temp_dir.path().join("config.toml");

        let args = InitArgs {
            dir: temp_dir.path().to_path_buf(),
            file: config_file.clone(),
            no_git: true,
            force: false,
        };

        assert!(init(args).is_ok());
        assert!(config_file.exists());
    }

    #[test]
    fn test_init_creates_git_repo() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("config.toml");

        let args = InitArgs {
            dir: temp_dir.path().to_path_buf(),
            file: config_file.clone(),
            no_git: false,
            force: false,
        };

        assert!(init(args).is_ok());
        assert!(config_file.exists());
        assert!(temp_dir.path().join(".git").exists());
    }

    #[test]
    fn test_init_does_not_overwrite_existing_file_without_force() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("config.toml");
        fs::write(&config_file, "existing content").unwrap();

        let args = InitArgs {
            dir: temp_dir.path().to_path_buf(),
            file: config_file.clone(),
            no_git: true,
            force: false,
        };

        assert!(init(args).is_err());
        assert_eq!(fs::read_to_string(config_file).unwrap(), "existing content");
    }

    #[test]
    fn test_init_overwrites_existing_file_with_force() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("config.toml");
        fs::write(&config_file, "old content").unwrap();

        let args = InitArgs {
            dir: temp_dir.path().to_path_buf(),
            file: config_file.clone(),
            no_git: true,
            force: true,
        };

        assert!(init(args).is_ok());
        assert_ne!(fs::read_to_string(config_file).unwrap(), "old content");
    }
}
