use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use tera;
use toml::Table;

pub fn read_context<P: AsRef<Path>>(answer_file: P) -> Result<tera::Context> {
    let answers = read_answers(answer_file)?;
    tera::Context::from_serialize(answers).context("Failed to serialize answers")
}

fn read_answers<P: AsRef<Path>>(answer_file: P) -> Result<Table> {
    let path = answer_file.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read answer file: {}", path.display()))?;
    toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML from {}", path.display()))
}
