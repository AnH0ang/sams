use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use toml;

use crate::args::GlobalArgs;

/// The configuration of the application
#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The file in which user parameter will be stored
    #[serde(default = "default_answer_file")]
    pub answer_file: PathBuf,

    /// The list of files to exclude when copying the template
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,

    /// The suffix of the template files
    #[serde(default = "default_template_suffix")]
    pub template_suffix: String,

    /// The suffix of the template files
    #[serde(default = "default_link_suffix")]
    pub link_suffix: String,

    /// Whether to respect the `.gitignore` file when copying the template
    #[serde(default = "default_respect_gitignore")]
    pub respect_gitignore: bool,

    /// The list of parameters to ask the user
    #[serde(default = "default_parameters")]
    pub parameters: Vec<Parameter>,

    /// List of install task to run
    #[serde(default = "default_tasks")]
    pub tasks: Vec<Task>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            answer_file: default_answer_file(),
            exclude: default_exclude(),
            template_suffix: default_template_suffix(),
            link_suffix: default_link_suffix(),
            respect_gitignore: default_respect_gitignore(),
            parameters: default_parameters(),
            tasks: default_tasks(),
        }
    }
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

fn default_link_suffix() -> String {
    "ln".to_string()
}

fn default_respect_gitignore() -> bool {
    true
}

fn default_parameters() -> Vec<Parameter> {
    Vec::new()
}

fn default_tasks() -> Vec<Task> {
    Vec::new()
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "lowercase")]
pub struct Task {
    /// The script to run
    pub script: PathBuf,

    /// The name of the task
    pub name: Option<String>,

    /// The working directory in which the command will be executed
    #[serde(default = "default_workdir")]
    pub workdir: PathBuf,

    /// The shell to use to run the command
    #[serde(default = "default_shell")]
    pub shell: String,
}

fn default_workdir() -> PathBuf {
    PathBuf::from(".")
}

fn default_shell() -> String {
    "sh".to_string()
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Int,
    Float,
    Str,
}

impl Config {
    pub fn from_args(global: &GlobalArgs) -> Result<Self> {
        Self::from_file(&global.root.join(&global.config_path))
    }

    pub fn from_file(file_path: &PathBuf) -> Result<Self> {
        let mut file = File::open(file_path)
            .with_context(|| format!("Failed to open config file at '{}'", file_path.display()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| format!("Failed to read config file at '{}'", file_path.display()))?;

        let config: Self = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file at '{}'", file_path.display()))?;

        Ok(config)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use toml::Table;

    use super::*;

    #[test]
    fn test_serialize_deserialize_config_with_select_parameter() {
        let config = Config {
            answer_file: PathBuf::from(".answers.toml"),
            exclude: vec![],
            template_suffix: "tpl".to_string(),
            link_suffix: "ln".to_string(),
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
            tasks: vec![],
        };

        // Expected readable YAML
        let expected_toml = r#"
answer_file = ".answers.toml"
exclude = []
template_suffix = "tpl"
link_suffix = "ln"
respect_gitignore = true
tasks = []

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
        let deserialized_config: Table = toml::from_str(&toml_string).unwrap();
        println!("{:?}", deserialized_config.get("answer_file").unwrap());
        // assert_eq!(config, deserialized_config);
    }
}
