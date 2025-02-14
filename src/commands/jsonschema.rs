use std::fs;

use anyhow::{Context, Result};
use schemars::schema_for;

use crate::args::JsonSchemaArgs;
use crate::config::Config;

pub fn generate_json_schema(args: JsonSchemaArgs) -> Result<()> {
    let schema = schema_for!(Config);
    let schema_content = serde_json::to_string_pretty(&schema)?;
    fs::write(args.file, schema_content).context("Failed to write json schema")?;
    Ok(())
}
