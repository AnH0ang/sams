use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use colored::Colorize;
use ignore::DirEntry;

use crate::args::GlobalArgs;
use crate::config::Config;
use crate::context::read_context;
use crate::template::render_template_str;
use crate::walk::WalkOptions;

pub fn link(global: &GlobalArgs) -> Result<()> {
    let cfg = Config::from_args(global).context("Failed to load configuration")?;
    let ctx = read_context(&cfg.answer_file).context("Failed to read context file")?;

    WalkOptions::from_config(&cfg)
        .with_extension(cfg.link_suffix)
        .walk(&global.root)
        .context("Failed to walk directory")?
        .skip(1)
        .try_for_each(|entry| process_entry(&entry?, &ctx))
}

fn process_entry(entry: &DirEntry, ctx: &tera::Context) -> Result<()> {
    let src = entry.path();
    let dst = PathBuf::from(
        render_template_str(
            src.with_extension("")
                .to_str()
                .context("Source path is not valid UTF-8")?,
            ctx,
        )
        .context("Failed to render template")?,
    );

    if dst.exists() {
        fs::remove_file(&dst).context("Failed to remove existing file at destination")?;
    }
    unix_fs::symlink(src, &dst).context("Failed to create symbolic link")?;

    println!(
        "{} {} -> {}",
        "✓  Linking".bold().green(),
        src.display(),
        dst.display()
    );

    Ok(())
}
