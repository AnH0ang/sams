use std::path::PathBuf;

use anyhow::Result;

use crate::args::{AskArgs, GlobalArgs, InstallArgs, LinkArgs, RenderArgs, SyncArgs};
use crate::commands::ask::ask;
use crate::commands::install::install;
use crate::commands::link::link;
use crate::commands::render::render;

pub fn sync(_args: SyncArgs, global: &GlobalArgs) -> Result<()> {
    // Ask
    let args = AskArgs { force: false };
    ask(args, global)?;

    // Link link
    let args = LinkArgs {
        path: PathBuf::from("."),
    };
    link(args, global)?;

    // Render templates
    let args = RenderArgs {
        path: PathBuf::from("."),
    };
    render(args, global)?;

    // Install
    let args = InstallArgs {};
    install(args, global)?;

    Ok(())
}
