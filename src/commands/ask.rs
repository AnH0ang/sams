use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result};
use inquire::{Select, Text};
use toml::{self, Value};

use crate::args::{AskArgs, GlobalArgs};
use crate::config::{read_config, DataType, Parameter};

fn parse_answer(input: &str, data_type: DataType) -> Result<Value> {
    match data_type {
        DataType::Float => input
            .parse::<f64>()
            .map(Value::Float)
            .with_context(|| format!("Failed to parse '{input}' as float")),
        DataType::Int => input
            .parse::<i64>()
            .map(Value::Integer)
            .with_context(|| format!("Failed to parse '{input}' as integer")),
        DataType::Str => Ok(Value::String(input.to_string())),
    }
}

pub fn ask(_args: AskArgs, global_args: GlobalArgs) -> Result<()> {
    let config = read_config(&global_args.config_path)?;

    // Ask the user for each parameter and store the answers.
    let mut answers: HashMap<String, Value> = HashMap::new();

    for parameter in config.parameters {
        match parameter {
            Parameter::Select {
                name,
                description,
                options,
            } => {
                let message = description.as_deref().unwrap_or(&name);
                let ans = Select::new(message, options)
                    .with_formatter(&|i| i.to_string())
                    .prompt()?;
                answers.insert(name, ans);
            },
            Parameter::Text {
                name,
                description,
                default,
                placeholder,
                data_type,
            } => {
                let message = description.as_deref().unwrap_or(&name);
                let mut prompt = Text::new(message);
                if let Some(default_msg) = default.as_deref() {
                    prompt = prompt.with_default(default_msg);
                }
                if let Some(placeholder_msg) = placeholder.as_deref() {
                    prompt = prompt.with_placeholder(placeholder_msg);
                }
                let ans = prompt.prompt()?;
                answers.insert(name, parse_answer(&ans, data_type)?);
            },
        }
    }

    // Serialize answers to TOML.
    let toml_string = toml::to_string(&answers)?;

    // Write the answers using std::fs::write (simpler than OpenOptions)
    fs::write(&config.answer_file, toml_string.as_bytes())
        .with_context(|| format!("Failed to write answers to file: {:?}", config.answer_file))?;

    Ok(())
}
