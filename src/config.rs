use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use toml::{self, Value};

/// The configuration of the application
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Config {
    /// The file in which user parameter will be stored
    #[serde(default = "default_answer_file")]
    pub answer_file: PathBuf,

    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,

    #[serde(default = "default_template_suffix")]
    pub template_suffix: String,

    #[serde(default = "default_respect_gitignore")]
    pub respect_gitignore: bool,

    /// The list of parameters to ask the user
    #[serde(default = "default_parameters")]
    pub parameters: Vec<Parameter>,
}

fn default_answer_file() -> PathBuf {
    PathBuf::from(".sams-answers.toml")
}

fn default_exclude() -> Vec<String> {
    Vec::new()
}

fn default_template_suffix() -> String {
    "tpl".to_string()
}

fn default_respect_gitignore() -> bool {
    true
}

fn default_parameters() -> Vec<Parameter> {
    Vec::new()
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Parameter {
    Select {
        /// The name of the parameter
        name: String,
        /// The description which will be displayed in the prompt
        description: Option<String>,
        /// A list of options to choose from
        options: Vec<Value>,
    },
    Text {
        /// The name of the parameter
        name: String,
        /// The description which will be displayed in the prompt
        description: Option<String>,
        /// The default value which will be used if the user does not provide any input
        default: Option<String>,
        /// A placeholder value which will be displayed in the prompt
        placeholder: Option<String>,
        /// The type of the user parameter
        #[serde(default = "default_data_type", rename = "type")]
        data_type: DataType,
    },
}

fn default_data_type() -> DataType {
    DataType::Str
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Int,
    Float,
    Str,
}

pub fn read_config(file_path: &PathBuf) -> Result<Config> {
    let mut file = File::open(file_path)
        .with_context(|| format!("Failed to open config file at '{}'", file_path.display()))?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .with_context(|| format!("Failed to read config file at '{}'", file_path.display()))?;

    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file at '{}'", file_path.display()))?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_config_with_select_parameter() {
        let config = Config {
            answer_file: PathBuf::from(".answers.toml"),
            exclude: vec![],
            template_suffix: "tpl".to_string(),
            respect_gitignore: true,
            parameters: vec![
                Parameter::Select {
                    name: "age".to_string(),
                    description: Some("Select your age".to_string()),
                    options: vec![Value::Integer(18), Value::Integer(25), Value::Integer(30)],
                },
                Parameter::Text {
                    name: "name".to_string(),
                    description: Some("Enter your name".to_string()),
                    default: None,
                    placeholder: None,
                    data_type: DataType::Str,
                },
            ],
        };

        // Expected readable YAML
        let expected_toml = r#"
answer_file = ".answers.toml"
exclude = []
template_suffix = "tpl"
respect_gitignore = true

[[parameters]]
kind = "select"
name = "age"
description = "Select your age"
options = [18, 25, 30]

[[parameters]]
kind = "text"
name = "name"
description = "Enter your name"
type = "str"
"#
        .trim();

        // Serialize to TOML
        let toml_string = toml::to_string(&config).unwrap();
        assert_eq!(toml_string.trim(), expected_toml);

        // Deserialize from TOML
        let deserialized_config: Value = toml::from_str(&toml_string).unwrap();
        println!("{:?}", deserialized_config.get("answer_file").unwrap());
        // assert_eq!(config, deserialized_config);
    }
}
