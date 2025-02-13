use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result};
use inquire::{Select, Text};
use toml;

use crate::args::{AskArgs, GlobalArgs};
use crate::config::{Config, DataType, Parameter, Value};

pub fn ask(args: AskArgs, global: &GlobalArgs) -> Result<()> {
    let cfg = Config::from_file(&global.config_path)?;

    let answer_file = global.root.join(&cfg.answer_file);

    if !args.force && answer_file.exists() {
        return Ok(());
    }

    // Ask the user for each parameter and store the answers.
    let mut answers: HashMap<String, Value> = HashMap::new();

    for param in cfg.parameters {
        match param {
            Parameter::Select {
                name,
                description,
                options,
            } => {
                let msg = description.as_deref().unwrap_or(&name);
                let ans = Select::new(msg, options)
                    .with_formatter(&|opt| opt.to_string())
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
                let msg = description.as_deref().unwrap_or(&name);
                let mut prompt = Text::new(msg);

                if let Some(default) = default.as_deref() {
                    prompt = prompt.with_default(default);
                }

                if let Some(placeholder) = placeholder.as_deref() {
                    prompt = prompt.with_placeholder(placeholder);
                }

                let ans = prompt.prompt()?;
                answers.insert(name, parse_input(&ans, data_type)?);
            },
        };
    }

    // Serialize answers to TOML.
    let toml_string = toml::to_string(&answers)?;

    // Write the answers using std::fs::write (simpler than OpenOptions)
    fs::write(&answer_file, toml_string.as_bytes())
        .with_context(|| format!("Failed to write answers to file: {:?}", answer_file))?;

    Ok(())
}

fn parse_input(input: &str, data_type: DataType) -> Result<Value> {
    match data_type {
        DataType::Float => input
            .parse::<f64>()
            .map(Value::Float)
            .with_context(|| format!("Failed to parse '{input}' as float")),
        DataType::Int => input
            .parse::<i64>()
            .map(Value::Integer)
            .with_context(|| format!("Failed to parse '{input}' as integer")),
        DataType::Str => Ok(Value::String(input.into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input() {
        assert_eq!(
            parse_input("42", DataType::Int).unwrap(),
            Value::Integer(42)
        );
        assert_eq!(
            parse_input("1.23", DataType::Float).unwrap(),
            Value::Float(1.23)
        );
        assert_eq!(
            parse_input("hello", DataType::Str).unwrap(),
            Value::String("hello".into())
        );
    }
}
