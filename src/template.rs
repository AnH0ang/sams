use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use tera::{Context as TeraContext, Tera};

pub fn render_template(from: &Path, to: &Path, context: &TeraContext) -> Result<()> {
    let template = fs::read_to_string(from)
        .with_context(|| format!("Failed to read template: {}", from.display()))?;

    let rendered = render_template_str(&template, context)?;

    fs::write(to, rendered).with_context(|| format!("Failed to write output: {}", to.display()))?;

    Ok(())
}

pub fn render_template_str(template: &str, context: &TeraContext) -> Result<String> {
    Tera::one_off(template, context, true).with_context(|| "Failed to render template")
}
