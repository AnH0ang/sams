use anyhow::Result;

use crate::args::{AskArgs, GlobalArgs};
use crate::commands::ask::ask;
use crate::commands::install::install;
use crate::commands::link::link;
use crate::commands::render::render;

pub fn sync(global: &GlobalArgs) -> Result<()> {
    // Ask
    let args = AskArgs { force: false };
    ask(args, global)?;

    // Link link
    link(global)?;

    // Render templates
    render(global)?;

    // Install
    install(global)?;

    Ok(())
}
