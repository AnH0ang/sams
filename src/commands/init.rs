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
    use std::fs;

    use tempfile::tempdir;

    use super::*;
    use crate::args::InitArgs;

    #[test]
    fn test_init_creates_config_and_git_repo_based_on_no_git() {
        let cases = vec![
            (true, false), // no_git=true: expect no .git
            (false, true), // no_git=false: expect .git
        ];

        for (no_git, expect_git) in cases {
            let temp_dir = tempdir().unwrap();
            let config_file = temp_dir.path().join("config.toml");

            let args = InitArgs {
                dir: temp_dir.path().to_path_buf(),
                file: config_file.clone(),
                no_git,
                force: false,
            };

            let result = init(args);
            assert!(result.is_ok(), "no_git={} should succeed", no_git);
            assert!(
                config_file.exists(),
                "config file should exist when no_git={}",
                no_git
            );

            let git_dir = temp_dir.path().join(".git");
            assert_eq!(
                git_dir.exists(),
                expect_git,
                "git repo existence check failed for no_git={}",
                no_git
            );
        }
    }

    #[test]
    fn test_force_flag_behavior() {
        let cases = vec![
            (false, false, false), // force=false: expect error and no overwrite
            (true, true, true),    // force=true: expect success and overwrite
        ];

        for (force, expect_success, expect_content_changed) in cases {
            let temp_dir = tempdir().unwrap();
            let config_file = temp_dir.path().join("config.toml");
            fs::write(&config_file, "old content").unwrap();

            let args = InitArgs {
                dir: temp_dir.path().to_path_buf(),
                file: config_file.clone(),
                no_git: true,
                force,
            };

            let result = init(args);
            assert_eq!(
                result.is_ok(),
                expect_success,
                "force={} should result in success={}",
                force,
                expect_success
            );

            let content = fs::read_to_string(&config_file).unwrap();
            if expect_content_changed {
                assert_ne!(
                    content, "old content",
                    "force={} should overwrite content",
                    force
                );
            } else {
                assert_eq!(
                    content, "old content",
                    "force={} should not overwrite content",
                    force
                );
            }
        }
    }
}
