use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::args::InitArgs;
use crate::config::{Config, DataType, Parameter};

pub fn init(args: InitArgs) -> Result<()> {
    let config = create_default_config();
    let toml_string = toml::to_string(&config)?;

    write_config_file(&args.file, &toml_string, args.force)
        .with_context(|| format!("Failed to initialize config file: {:?}", args.file))?;

    Ok(())
}

fn create_default_config() -> Config {
    Config {
        parameters: vec![Parameter::Text {
            name: "user".to_string(),
            description: Some("Enter username".to_string()),
            default: None,
            placeholder: Some("username".to_string()),
            data_type: DataType::Str,
        }],
        ..Config::default()
    }
}

fn write_config_file(path: &PathBuf, content: &str, force: bool) -> Result<()> {
    if !force && path.exists() {
        anyhow::bail!("File already exists: {:?}", path);
    }

    fs::write(path, content)
        .with_context(|| format!("Failed to write config to file: {:?}", path))?;

    Ok(())
}
