use std::process::Command;

use anyhow::Result;

use crate::args::{CloneArgs, GlobalArgs};
use crate::commands::sync::sync;

pub fn clone(args: CloneArgs, global: GlobalArgs) -> Result<()> {
    // Clone
    Command::new("git")
        .arg("clone")
        .arg(&args.url)
        .arg(&args.dest)
        .spawn()?
        .wait()?;

    // Sync
    let global = GlobalArgs {
        root: args.dest.clone(),
        ..global
    };
    sync(&global)?;

    Ok(())
}
