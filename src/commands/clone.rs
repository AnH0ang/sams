use std::process::Command;

use anyhow::Result;

use crate::args::{CloneArgs, GlobalArgs, SyncArgs};
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
    sync(
        SyncArgs { ask: true },
        &GlobalArgs {
            root: args.dest.clone(),
            ..global
        },
    )?;

    Ok(())
}
