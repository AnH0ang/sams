use std::process::Command;

use anyhow::Result;

use crate::args::GlobalArgs;

pub fn pull(global: &GlobalArgs) -> Result<()> {
    Command::new("git")
        .arg("pull")
        .current_dir(&global.root)
        .spawn()?
        .wait()?;

    Ok(())
}
